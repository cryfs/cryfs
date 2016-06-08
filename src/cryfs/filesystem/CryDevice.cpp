#include <blockstore/implementations/caching/CachingBlockStore.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "parallelaccessfsblobstore/DirBlobRef.h"
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"

#include <fspp/fuse/FuseErrnoException.h>
#include <blobstore/implementations/onblocks/BlobStoreOnBlocks.h>
#include <blobstore/implementations/onblocks/BlobOnBlocks.h>
#include <blockstore/implementations/encrypted/EncryptedBlockStore.h>
#include "parallelaccessfsblobstore/ParallelAccessFsBlobStore.h"
#include "cachingfsblobstore/CachingFsBlobStore.h"
#include "../config/CryCipher.h"

using std::string;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blockstore::BlockStore;
using blockstore::Key;
using blockstore::encrypted::EncryptedBlockStore;
using blobstore::onblocks::BlobStoreOnBlocks;
using blobstore::onblocks::BlobOnBlocks;
using blockstore::caching::CachingBlockStore;
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

CryDevice::CryDevice(CryConfigFile configFile, unique_ref<BlockStore> blockStore)
: _fsBlobStore(
      make_unique_ref<ParallelAccessFsBlobStore>(
        make_unique_ref<CachingFsBlobStore>(
          make_unique_ref<FsBlobStore>(
            make_unique_ref<BlobStoreOnBlocks>(
              make_unique_ref<CachingBlockStore>(
                CreateEncryptedBlockStore(*configFile.config(), std::move(blockStore))
              ), configFile.config()->BlocksizeBytes())))
        )
      ),
  _rootKey(GetOrCreateRootKey(&configFile)),
  _onFsAction() {
}

Key CryDevice::CreateRootBlobAndReturnKey() {
  auto rootBlob =  _fsBlobStore->createDirBlob();
  rootBlob->flush(); // Don't cache, but directly write the root blob (this causes it to fail early if the base directory is not accessible)
  return rootBlob->key();
}

optional<unique_ref<fspp::Node>> CryDevice::Load(const bf::path &path) {
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
    LOG(ERROR) << "Could not load root blob. Is the base directory accessible?";
    throw FuseErrnoException(EIO);
  }
  unique_ref<FsBlobRef> currentBlob = std::move(*currentBlobOpt);

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
}

unique_ref<FileBlobRef> CryDevice::CreateFileBlob() {
  return _fsBlobStore->createFileBlob();
}

unique_ref<DirBlobRef> CryDevice::CreateDirBlob() {
  return _fsBlobStore->createDirBlob();
}

unique_ref<SymlinkBlobRef> CryDevice::CreateSymlinkBlob(const bf::path &target) {
  return _fsBlobStore->createSymlinkBlob(target);
}

unique_ref<FsBlobRef> CryDevice::LoadBlob(const blockstore::Key &key) {
  auto blob = _fsBlobStore->load(key);
  if (blob == none) {
    LOG(ERROR) << "Could not load blob " << key.ToString() << ". Is the base directory accessible?";
    throw FuseErrnoException(EIO);
  }
  return std::move(*blob);
}

void CryDevice::RemoveBlob(const blockstore::Key &key) {
  auto blob = _fsBlobStore->load(key);
  if (blob == none) {
    LOG(ERROR) << "Could not load blob " << key.ToString() << ". Is the base directory accessible?";
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

cpputils::unique_ref<blockstore::BlockStore> CryDevice::CreateEncryptedBlockStore(const CryConfig &config, unique_ref<BlockStore> baseBlockStore) {
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
