#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"

#include <fspp/fs_interface/FuseErrnoException.h>
#include <blobstore/implementations/onblocks/BlobStoreOnBlocks.h>
#include <blobstore/implementations/onblocks/BlobOnBlocks.h>
#include <blockstore/implementations/rustbridge/RustBlockStore.h>
#include <blockstore/implementations/rustbridge/CxxCallback.h>
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/ParallelAccessFsBlobStore.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/CachingFsBlobStore.h"
#include "cryfs/impl/config/CryCipher.h"
#include <cpp-utils/system/homedir.h>
#include <gitversion/VersionCompare.h>
#include <blockstore/interface/BlockStore2.h>
#include "cryfs/impl/localstate/LocalStateDir.h"
#include <cryfs/impl/CryfsException.h>


using std::string;


//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::FuseErrnoException;

using blockstore::BlockStore2;
using blockstore::BlockId;
using blockstore::rust::CxxCallback;
using blobstore::BlobStore;
using blobstore::onblocks::BlobStoreOnBlocks;
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

CryDevice::CryDevice(std::shared_ptr<CryConfigFile> configFile, const boost::filesystem::path& basedir, const LocalStateDir& localStateDir, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation)
: _fsBlobStore(CreateFsBlobStore(
    CreateBlobStore(basedir, localStateDir, configFile.get(), myClientId, allowIntegrityViolations, missingBlockIsIntegrityViolation, std::move(onIntegrityViolation)),
    configFile.get()
  )),
  _rootBlobId(GetOrCreateRootBlobId(configFile.get())), _configFile(std::move(configFile)),
  _onFsAction() {
}

CryDevice::CryDevice(std::shared_ptr<CryConfigFile> configFile, const LocalStateDir& localStateDir, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation)
: _fsBlobStore(CreateFsBlobStore(
    CreateFakeBlobStore(localStateDir, configFile.get(), myClientId, allowIntegrityViolations, missingBlockIsIntegrityViolation, std::move(onIntegrityViolation)),
    configFile.get()
  )),
  _rootBlobId(GetOrCreateRootBlobId(configFile.get())), _configFile(std::move(configFile)),
  _onFsAction() {
}

unique_ref<parallelaccessfsblobstore::ParallelAccessFsBlobStore> CryDevice::CreateFsBlobStore(unique_ref<blobstore::BlobStore> blobStore, CryConfigFile *configFile) {
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
  string rootBlobId = configFile->config()->RootBlob();
  if ("" == rootBlobId) {
    return make_unique_ref<FsBlobStore>(std::move(blobStore));
  }
  if (!configFile->config()->HasParentPointers()) {
    auto result = FsBlobStore::migrate(std::move(blobStore), BlockId::FromString(rootBlobId));
    // Don't migrate again if it was successful
    configFile->config()->SetHasParentPointers(true);
    configFile->save();
    return result;
  }
  return make_unique_ref<FsBlobStore>(std::move(blobStore));
}
#endif

unique_ref<blobstore::BlobStore> CryDevice::CreateBlobStore(const boost::filesystem::path& basedir, const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation) {
  auto statePath = localStateDir.forFilesystemId(configFile->config()->FilesystemId());
  auto integrityFilePath = statePath / "integritydata";
  return make_unique_ref<BlobStoreOnBlocks>(
    make_unique_ref<blockstore::rust::RustBlockStore>(
      blockstore::rust::bridge::new_locking_integrity_encrypted_ondisk_blockstore(integrityFilePath.c_str(), myClientId, allowIntegrityViolations, missingBlockIsIntegrityViolation, std::make_unique<CxxCallback>(onIntegrityViolation), configFile->config()->Cipher(), configFile->config()->EncryptionKey(), basedir.c_str())
    ),
    configFile->config()->BlocksizeBytes());
}

unique_ref<blobstore::BlobStore> CryDevice::CreateFakeBlobStore(const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation) {
  auto statePath = localStateDir.forFilesystemId(configFile->config()->FilesystemId());
  auto integrityFilePath = statePath / "integritydata";
  return make_unique_ref<BlobStoreOnBlocks>(
    make_unique_ref<blockstore::rust::RustBlockStore>(
      blockstore::rust::bridge::new_locking_integrity_encrypted_inmemory_blockstore(integrityFilePath.c_str(), myClientId, allowIntegrityViolations, missingBlockIsIntegrityViolation, std::make_unique<CxxCallback>(onIntegrityViolation), configFile->config()->Cipher(), configFile->config()->EncryptionKey())
    ),
    configFile->config()->BlocksizeBytes());
}

// unique_ref<BlockStore2> CryDevice::CreateIntegrityEncryptedBlockStore(unique_ref<BlockStore2> blockStore, const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation) {
//   auto encryptedBlockStore = CreateEncryptedBlockStore(*configFile->config(), std::move(blockStore));
//   auto statePath = localStateDir.forFilesystemId(configFile->config()->FilesystemId());
//   auto integrityFilePath = statePath / "integritydata";
//
// #ifndef CRYFS_NO_COMPATIBILITY
//   if (!configFile->config()->HasVersionNumbers()) {
//     IntegrityBlockStore2::migrateFromBlockstoreWithoutVersionNumbers(encryptedBlockStore.get(), integrityFilePath, myClientId);
//     configFile->config()->SetBlocksizeBytes(configFile->config()->BlocksizeBytes() + IntegrityBlockStore2::HEADER_LENGTH - blockstore::BlockId::BINARY_LENGTH); // Minus BlockId size because EncryptedBlockStore doesn't store the BlockId anymore (that was moved to IntegrityBlockStore)
//     // Don't migrate again if it was successful
//     configFile->config()->SetHasVersionNumbers(true);
//     configFile->save();
//   }
// #endif
//
//   try {
//     return make_unique_ref<IntegrityBlockStore2>(std::move(encryptedBlockStore), integrityFilePath, myClientId,
//                                                  allowIntegrityViolations, missingBlockIsIntegrityViolation,
//                                                  std::move(onIntegrityViolation));
//   } catch (const blockstore::integrity::IntegrityViolationOnPreviousRun& e) {
//     throw CryfsException(string() +
//                         "There was an integrity violation detected. Preventing any further access to the file system. " +
//                         "This can either happen if an attacker changed your files or rolled back the file system to a previous state, " +
//                         "but it can also happen if you rolled back the file system yourself, for example restored a backup. " +
//                         "If you want to reset the integrity data (i.e. accept changes made by a potential attacker), " +
//                         "please delete the following file before re-mounting it: " +
//                         e.stateFile().string(), ErrorCode::IntegrityViolationOnPreviousRun);
//   }
// }

BlockId CryDevice::CreateRootBlobAndReturnId() {
  auto rootBlob =  _fsBlobStore->createDirBlob(blockstore::BlockId::Null());
  rootBlob->flush(); // Don't cache, but directly write the root blob (this causes it to fail early if the base directory is not accessible)
  return rootBlob->blockId();
}

const CryConfig &CryDevice::config() const {
  return *_configFile->config();
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
  ASSERT(path.has_root_directory() && !path.has_root_name(), "Must be an absolute path (but on windows without device specifier): " + path.string());

  callFsActionCallbacks();

  if (path.parent_path().empty()) {
    //We are asked to load the base directory '/'.
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, none, none, _rootBlobId));
  }

  auto parentWithAncestors = LoadDirBlobWithAncestors(path.parent_path(), [](const BlockId&){});
  if (parentWithAncestors == none) {
    return none;
  }
  auto parent = std::move(parentWithAncestors->blob);
  auto grandparent = std::move(parentWithAncestors->parent);

  auto optEntry = parent->GetChild(path.filename().string());
  if (optEntry == boost::none) {
    return boost::none;
  }
  const auto &entry = *optEntry;

  switch(entry.type()) {
    case fspp::Dir::EntryType::DIR:
      return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, std::move(parent), std::move(grandparent), entry.blockId()));
    case fspp::Dir::EntryType::FILE:
      return optional<unique_ref<fspp::Node>>(make_unique_ref<CryFile>(this, std::move(parent), std::move(grandparent), entry.blockId()));
    case  fspp::Dir::EntryType::SYMLINK:
	  return optional<unique_ref<fspp::Node>>(make_unique_ref<CrySymlink>(this, std::move(parent), std::move(grandparent), entry.blockId()));
  }
  ASSERT(false, "Switch/case not exhaustive");
}

optional<CryDevice::DirBlobWithAncestors> CryDevice::LoadDirBlobWithAncestors(const bf::path &path, std::function<void (const blockstore::BlockId&)> ancestor_callback) {
  auto blob = LoadBlobWithAncestors(path, std::move(ancestor_callback));
  if (blob == none) {
    return none;
  }
  auto dir = dynamic_pointer_move<DirBlobRef>(blob->blob);
  if (dir == none) {
    throw FuseErrnoException(ENOTDIR); // Loaded blob is not a directory
  }
  return DirBlobWithAncestors{std::move(*dir), std::move(blob->parent)};
}

optional<CryDevice::BlobWithAncestors> CryDevice::LoadBlobWithAncestors(const bf::path &path, std::function<void (const blockstore::BlockId&)> ancestor_callback) {
  optional<unique_ref<DirBlobRef>> parentBlob = none;
  optional<unique_ref<FsBlobRef>> currentBlobOpt = _fsBlobStore->load(_rootBlobId);
  if (currentBlobOpt == none) {
    LOG(ERR, "Could not load root blob. Is the base directory accessible?");
    throw FuseErrnoException(EIO);
  }
  unique_ref<FsBlobRef> currentBlob = std::move(*currentBlobOpt);
  ASSERT(currentBlob->parentPointer() == BlockId::Null(), "Root Blob should have a nullptr as parent");

  for (const bf::path &component : path.relative_path()) {
    ancestor_callback(currentBlob->blockId());
    auto currentDir = dynamic_pointer_move<DirBlobRef>(currentBlob);
    if (currentDir == none) {
      throw FuseErrnoException(ENOTDIR); // Path component is not a dir
    }

    auto childOpt = (*currentDir)->GetChild(component.string());
    if (childOpt == boost::none) {
      // Child entry in directory not found
      return none;
    }
    BlockId childId = childOpt->blockId();
    auto nextBlob = _fsBlobStore->load(childId);
    if (nextBlob == none) {
      throw FuseErrnoException(EIO); // Blob for directory entry not found
    }
    parentBlob = std::move(*currentDir);
    currentBlob = std::move(*nextBlob);
    ASSERT(currentBlob->parentPointer() == (*parentBlob)->blockId(), "Blob has wrong parent pointer");
  }

  return BlobWithAncestors{std::move(currentBlob), std::move(parentBlob)};

  //TODO (I think this is resolved, but I should test it)
  //     Running the python script, waiting for "Create files in sequential order...", then going into dir ~/tmp/cryfs-mount-.../Bonnie.../ and calling "ls"
  //     crashes cryfs with a sigsegv.
  //     Possible reason: Many parallel changes to a directory blob are a race condition. Need something like ParallelAccessStore!
}

CryDevice::statvfs CryDevice::statfs() {
  callFsActionCallbacks();

  uint64_t numUsedBlocks = _fsBlobStore->numBlocks();
  uint64_t numFreeBlocks = _fsBlobStore->estimateSpaceForNumBlocksLeft();

  statvfs result;
  result.max_filename_length = 255; // We theoretically support unlimited file name length, but this is default for many Linux file systems, so probably also makes sense for CryFS.

  result.blocksize = _fsBlobStore->virtualBlocksizeBytes();
  result.num_total_blocks = numUsedBlocks + numFreeBlocks;
  result.num_free_blocks = numFreeBlocks;
  result.num_available_blocks = numFreeBlocks;

  result.num_total_inodes = numUsedBlocks + numFreeBlocks;
  result.num_free_inodes = numFreeBlocks;
  result.num_available_inodes = numFreeBlocks;

  return result;
}

unique_ref<FileBlobRef> CryDevice::CreateFileBlob(const blockstore::BlockId &parent) {
  return _fsBlobStore->createFileBlob(parent);
}

unique_ref<DirBlobRef> CryDevice::CreateDirBlob(const blockstore::BlockId &parent) {
  return _fsBlobStore->createDirBlob(parent);
}

unique_ref<SymlinkBlobRef> CryDevice::CreateSymlinkBlob(const bf::path &target, const blockstore::BlockId &parent) {
  return _fsBlobStore->createSymlinkBlob(target, parent);
}

unique_ref<FsBlobRef> CryDevice::LoadBlob(const blockstore::BlockId &blockId) {
  auto blob = _fsBlobStore->load(blockId);
  if (blob == none) {
    LOG(ERR, "Could not load blob {}. Is the base directory accessible?", blockId.ToString());
    throw FuseErrnoException(EIO);
  }
  return std::move(*blob);
}

void CryDevice::RemoveBlob(const blockstore::BlockId &blockId) {
  auto blob = _fsBlobStore->load(blockId);
  if (blob == none) {
    LOG(ERR, "Could not load blob {}. Is the base directory accessible?", blockId.ToString());
    throw FuseErrnoException(EIO);
  }
  _fsBlobStore->remove(std::move(*blob));
}

BlockId CryDevice::GetOrCreateRootBlobId(CryConfigFile *configFile) {
  string root_blockId = configFile->config()->RootBlob();
  if (root_blockId == "") { // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
    auto new_blockId = CreateRootBlobAndReturnId();
    configFile->config()->SetRootBlob(new_blockId.ToString());
    configFile->save();
    return new_blockId;
  }

  return BlockId::FromString(root_blockId);
}

// cpputils::unique_ref<blockstore::BlockStore2> CryDevice::CreateEncryptedBlockStore(const CryConfig &config, unique_ref<BlockStore2> baseBlockStore) {
//   //TODO Test that CryFS is using the specified cipher
//   return CryCiphers::find(config.Cipher()).createEncryptedBlockstore(std::move(baseBlockStore), config.EncryptionKey());
// }

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
