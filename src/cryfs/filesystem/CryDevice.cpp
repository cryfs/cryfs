#include <blockstore/implementations/caching/CachingBlockStore2.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "parallelaccessfsblobstore/DirBlobRef.h"
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"

#include <fspp/fuse/FuseErrnoException.h>
#include <blobstore/implementations/onblocks/BlobStoreOnBlocks.h>
#include <blobstore/implementations/onblocks/BlobOnBlocks.h>
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include <blockstore/implementations/encrypted/EncryptedBlockStore2.h>
#include <blockstore/implementations/integrity/IntegrityBlockStore2.h>
#include "parallelaccessfsblobstore/ParallelAccessFsBlobStore.h"
#include "cachingfsblobstore/CachingFsBlobStore.h"
#include "../config/CryCipher.h"
#include <cpp-utils/system/homedir.h>
#include <gitversion/VersionCompare.h>
#include <blockstore/interface/BlockStore2.h>
#include "cryfs/localstate/MyClientId.h"
#include "cryfs/localstate/LocalStateDir.h"

using std::string;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::Key;
using blockstore::encrypted::EncryptedBlockStore2;
using blobstore::BlobStore;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blobstore::onblocks::BlobStoreOnBlocks;
using blobstore::onblocks::BlobOnBlocks;
using blockstore::caching::CachingBlockStore2;
using blockstore::integrity::IntegrityBlockStore2;
using gitversion::VersionCompare;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;
using cryfs::fsblobstore::FsBlobStore;
using cryfs::cachingfsblobstore::CachingFsBlobStore;
using cryfs::parallelaccessfsblobstore::ParallelAccessFsBlobStore;
using cryfs::parallelaccessfsblobstore::FileBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;
using cryfs::parallelaccessfsblobstore::SymlinkBlobRef;
using cryfs::parallelaccessfsblobstore::FsBlobRef;
using namespace cpputils::logging;

namespace bf = boost::filesystem;

namespace cryfs {

CryDevice::CryDevice(CryConfigFile configFile, unique_ref<BlockStore2> blockStore, uint32_t myClientId, bool noIntegrityChecks)
: _fsBlobStore(CreateFsBlobStore(std::move(blockStore), &configFile, myClientId, noIntegrityChecks)),
  _rootKey(GetOrCreateRootKey(&configFile)),
  _onFsAction() {
}

unique_ref<parallelaccessfsblobstore::ParallelAccessFsBlobStore> CryDevice::CreateFsBlobStore(unique_ref<BlockStore2> blockStore, CryConfigFile *configFile, uint32_t myClientId, bool noIntegrityChecks) {
  auto blobStore = CreateBlobStore(std::move(blockStore), configFile, myClientId, noIntegrityChecks);

#ifndef CRYFS_NO_COMPATIBILITY
  auto fsBlobStore = MigrateOrCreateFsBlobStore(std::move(blobStore), configFile);
#else
  auto fsBlobStore = make_unique_ref<FsBlobStore>(std::move(blobStore));
#endif

  return make_unique_ref<ParallelAccessFsBlobStore>(
    make_unique_ref<CachingFsBlobStore>(
      std::move(fsBlobStore)
    )
  );
}

#ifndef CRYFS_NO_COMPATIBILITY
unique_ref<fsblobstore::FsBlobStore> CryDevice::MigrateOrCreateFsBlobStore(unique_ref<BlobStore> blobStore, CryConfigFile *configFile) {
  string rootBlobKey = configFile->config()->RootBlob();
  if ("" == rootBlobKey) {
    return make_unique_ref<FsBlobStore>(std::move(blobStore));
  }
  return FsBlobStore::migrateIfNeeded(std::move(blobStore), Key::FromString(rootBlobKey));
}
#endif

unique_ref<blobstore::BlobStore> CryDevice::CreateBlobStore(unique_ref<BlockStore2> blockStore, CryConfigFile *configFile, uint32_t myClientId, bool noIntegrityChecks) {
  auto integrityEncryptedBlockStore = CreateIntegrityEncryptedBlockStore(std::move(blockStore), configFile, myClientId, noIntegrityChecks);
  // Create integrityEncryptedBlockStore not in the same line as BlobStoreOnBlocks, because it can modify BlocksizeBytes
  // in the configFile and therefore has to be run before the second parameter to the BlobStoreOnBlocks parameter is evaluated.
  return make_unique_ref<BlobStoreOnBlocks>(
     make_unique_ref<LowToHighLevelBlockStore>(
         make_unique_ref<CachingBlockStore2>(
             std::move(integrityEncryptedBlockStore)
         )
     ),
     configFile->config()->BlocksizeBytes());
}

unique_ref<BlockStore2> CryDevice::CreateIntegrityEncryptedBlockStore(unique_ref<BlockStore2> blockStore, CryConfigFile *configFile, uint32_t myClientId, bool noIntegrityChecks) {
  auto encryptedBlockStore = CreateEncryptedBlockStore(*configFile->config(), std::move(blockStore));
  auto statePath = LocalStateDir::forFilesystemId(configFile->config()->FilesystemId());
  auto integrityFilePath = statePath / "integritydata";

#ifndef CRYFS_NO_COMPATIBILITY
  if (!configFile->config()->HasVersionNumbers()) {
    IntegrityBlockStore2::migrateFromBlockstoreWithoutVersionNumbers(encryptedBlockStore.get(), integrityFilePath, myClientId);
    configFile->config()->SetBlocksizeBytes(configFile->config()->BlocksizeBytes() + IntegrityBlockStore2::HEADER_LENGTH);
    configFile->config()->SetHasVersionNumbers(true);
    configFile->save();
  }
#endif
    
  return make_unique_ref<IntegrityBlockStore2>(std::move(encryptedBlockStore), integrityFilePath, myClientId, noIntegrityChecks, false);
}

Key CryDevice::CreateRootBlobAndReturnKey() {
  auto rootBlob =  _fsBlobStore->createDirBlob(blockstore::Key::Null());
  rootBlob->flush(); // Don't cache, but directly write the root blob (this causes it to fail early if the base directory is not accessible)
  return rootBlob->key();
}

optional<unique_ref<fspp::File>> CryDevice::LoadFile(const bf::path &path) {
  auto loaded = Load(path);
  if (loaded == none) {
    return none;
  }
  auto file = cpputils::dynamic_pointer_move<fspp::File>(*loaded);
  if (file == none) {
    throw fspp::fuse::FuseErrnoException(EISDIR); // TODO Also EISDIR if it is a symlink?
  }
  return std::move(*file);
}

optional<unique_ref<fspp::Dir>> CryDevice::LoadDir(const bf::path &path) {
  auto loaded = Load(path);
  if (loaded == none) {
    return none;
  }
  auto dir = cpputils::dynamic_pointer_move<fspp::Dir>(*loaded);
  if (dir == none) {
    throw fspp::fuse::FuseErrnoException(ENOTDIR);
  }
  return std::move(*dir);
}

optional<unique_ref<fspp::Symlink>> CryDevice::LoadSymlink(const bf::path &path) {
  auto loaded = Load(path);
  if (loaded == none) {
    return none;
  }
  auto lnk = cpputils::dynamic_pointer_move<fspp::Symlink>(*loaded);
  if (lnk == none) {
    throw fspp::fuse::FuseErrnoException(ENOTDIR); // TODO ENOTDIR although it is a symlink?
  }
  return std::move(*lnk);
}

optional<unique_ref<fspp::Node>> CryDevice::Load(const bf::path &path) {
  // TODO Is it faster to not let CryFile/CryDir/CryDevice inherit from CryNode and loading CryNode without having to know what it is?
  // TODO Split into smaller functions
  ASSERT(path.is_absolute(), "Non absolute path given");

  callFsActionCallbacks();

  if (path.parent_path().empty()) {
    //We are asked to load the base directory '/'.
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, none, none, _rootKey));
  }

  auto parentWithGrandparent = LoadDirBlobWithParent(path.parent_path());
  auto parent = std::move(parentWithGrandparent.blob);
  auto grandparent = std::move(parentWithGrandparent.parent);

  auto optEntry = parent->GetChild(path.filename().native());
  if (optEntry == boost::none) {
    return boost::none;
  }
  const auto &entry = *optEntry;

  switch(entry.type()) {
    case fspp::Dir::EntryType::DIR:
      return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, std::move(parent), std::move(grandparent), entry.key()));
    case fspp::Dir::EntryType::FILE:
      return optional<unique_ref<fspp::Node>>(make_unique_ref<CryFile>(this, std::move(parent), std::move(grandparent), entry.key()));
    case  fspp::Dir::EntryType::SYMLINK:
	  return optional<unique_ref<fspp::Node>>(make_unique_ref<CrySymlink>(this, std::move(parent), std::move(grandparent), entry.key()));
  }
  ASSERT(false, "Switch/case not exhaustive");
}

CryDevice::DirBlobWithParent CryDevice::LoadDirBlobWithParent(const bf::path &path) {
  auto blob = LoadBlobWithParent(path);
  auto dir = dynamic_pointer_move<DirBlobRef>(blob.blob);
  if (dir == none) {
    throw FuseErrnoException(ENOTDIR); // Loaded blob is not a directory
  }
  return DirBlobWithParent{std::move(*dir), std::move(blob.parent)};
}

CryDevice::BlobWithParent CryDevice::LoadBlobWithParent(const bf::path &path) {
  optional<unique_ref<DirBlobRef>> parentBlob = none;
  optional<unique_ref<FsBlobRef>> currentBlobOpt = _fsBlobStore->load(_rootKey);
  if (currentBlobOpt == none) {
    LOG(ERROR, "Could not load root blob. Is the base directory accessible?");
    throw FuseErrnoException(EIO);
  }
  unique_ref<FsBlobRef> currentBlob = std::move(*currentBlobOpt);
  ASSERT(currentBlob->parentPointer() == Key::Null(), "Root Blob should have a nullptr as parent");

  for (const bf::path &component : path.relative_path()) {
    auto currentDir = dynamic_pointer_move<DirBlobRef>(currentBlob);
    if (currentDir == none) {
      throw FuseErrnoException(ENOTDIR); // Path component is not a dir
    }

    auto childOpt = (*currentDir)->GetChild(component.c_str());
    if (childOpt == boost::none) {
      throw FuseErrnoException(ENOENT); // Child entry in directory not found
    }
    Key childKey = childOpt->key();
    auto nextBlob = _fsBlobStore->load(childKey);
    if (nextBlob == none) {
      throw FuseErrnoException(ENOENT); // Blob for directory entry not found
    }
    parentBlob = std::move(*currentDir);
    currentBlob = std::move(*nextBlob);
    ASSERT(currentBlob->parentPointer() == (*parentBlob)->key(), "Blob has wrong parent pointer");
  }

  return BlobWithParent{std::move(currentBlob), std::move(parentBlob)};

  //TODO (I think this is resolved, but I should test it)
  //     Running the python script, waiting for "Create files in sequential order...", then going into dir ~/tmp/cryfs-mount-.../Bonnie.../ and calling "ls"
  //     crashes cryfs with a sigsegv.
  //     Possible reason: Many parallel changes to a directory blob are a race condition. Need something like ParallelAccessStore!
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  // TODO Do we need path for something? What does it represent from fuse side?
  UNUSED(path);
  callFsActionCallbacks();
  uint64_t numUsedBlocks = _fsBlobStore->numBlocks();
  uint64_t numFreeBlocks = _fsBlobStore->estimateSpaceForNumBlocksLeft();
  fsstat->f_bsize = _fsBlobStore->virtualBlocksizeBytes();
  fsstat->f_blocks = numUsedBlocks + numFreeBlocks;
  fsstat->f_bfree = numFreeBlocks;
  fsstat->f_bavail = numFreeBlocks;
  fsstat->f_files = numUsedBlocks + numFreeBlocks;
  fsstat->f_ffree = numFreeBlocks;
  fsstat->f_namemax = 255; // We theoretically support unlimited file name length, but this is default for many Linux file systems, so probably also makes sense for CryFS.
  //f_frsize, f_favail, f_fsid and f_flag are ignored in fuse, see http://fuse.sourcearchive.com/documentation/2.7.0/structfuse__operations_4e765e29122e7b6b533dc99849a52655.html#4e765e29122e7b6b533dc99849a52655
  fsstat->f_frsize = fsstat->f_bsize; // even though this is supposed to be ignored, osxfuse needs it.
}

unique_ref<FileBlobRef> CryDevice::CreateFileBlob(const blockstore::Key &parent) {
  return _fsBlobStore->createFileBlob(parent);
}

unique_ref<DirBlobRef> CryDevice::CreateDirBlob(const blockstore::Key &parent) {
  return _fsBlobStore->createDirBlob(parent);
}

unique_ref<SymlinkBlobRef> CryDevice::CreateSymlinkBlob(const bf::path &target, const blockstore::Key &parent) {
  return _fsBlobStore->createSymlinkBlob(target, parent);
}

unique_ref<FsBlobRef> CryDevice::LoadBlob(const blockstore::Key &key) {
  auto blob = _fsBlobStore->load(key);
  if (blob == none) {
    LOG(ERROR, "Could not load blob {}. Is the base directory accessible?", key.ToString());
    throw FuseErrnoException(EIO);
  }
  return std::move(*blob);
}

void CryDevice::RemoveBlob(const blockstore::Key &key) {
  auto blob = _fsBlobStore->load(key);
  if (blob == none) {
    LOG(ERROR, "Could not load blob. Is the base directory accessible?", key.ToString());
    throw FuseErrnoException(EIO);
  }
  _fsBlobStore->remove(std::move(*blob));
}

Key CryDevice::GetOrCreateRootKey(CryConfigFile *configFile) {
  string root_key = configFile->config()->RootBlob();
  if (root_key == "") {
    auto new_key = CreateRootBlobAndReturnKey();
    configFile->config()->SetRootBlob(new_key.ToString());
    configFile->save();
    return new_key;
  }

  return Key::FromString(root_key);
}

cpputils::unique_ref<blockstore::BlockStore2> CryDevice::CreateEncryptedBlockStore(const CryConfig &config, unique_ref<BlockStore2> baseBlockStore) {
  //TODO Test that CryFS is using the specified cipher
  return CryCiphers::find(config.Cipher()).createEncryptedBlockstore(std::move(baseBlockStore), config.EncryptionKey());
}

void CryDevice::onFsAction(std::function<void()> callback) {
  _onFsAction.push_back(callback);
}

void CryDevice::callFsActionCallbacks() const {
  for (const auto &callback : _onFsAction) {
    callback();
  }
}

uint64_t CryDevice::numBlocks() const {
  return _fsBlobStore->numBlocks();
}

}
