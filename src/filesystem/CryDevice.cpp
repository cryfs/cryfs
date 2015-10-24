#include <messmer/blockstore/implementations/caching/CachingBlockStore.h>
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>
#include "parallelaccessfsblobstore/DirBlobRef.h"
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "messmer/blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "messmer/blobstore/implementations/onblocks/BlobOnBlocks.h"
#include "messmer/blockstore/implementations/encrypted/EncryptedBlockStore.h"
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
using blockstore::encrypted::AES256_CFB;
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

namespace bf = boost::filesystem;

namespace cryfs {

constexpr uint32_t CryDevice::BLOCKSIZE_BYTES;

CryDevice::CryDevice(CryConfigFile configFile, unique_ref<BlockStore> blockStore)
: _fsBlobStore(
      make_unique_ref<ParallelAccessFsBlobStore>(
        make_unique_ref<CachingFsBlobStore>(
          make_unique_ref<FsBlobStore>(
            make_unique_ref<BlobStoreOnBlocks>(
              make_unique_ref<CachingBlockStore>(
                CreateEncryptedBlockStore(*configFile.config(), std::move(blockStore))
              ), BLOCKSIZE_BYTES)))
        )
      ),
  _rootKey(GetOrCreateRootKey(&configFile)) {
}

Key CryDevice::CreateRootBlobAndReturnKey() {
  return _fsBlobStore->createDirBlob()->key();
}

optional<unique_ref<fspp::Node>> CryDevice::Load(const bf::path &path) {
  ASSERT(path.is_absolute(), "Non absolute path given");

  if (path.parent_path().empty()) {
    //We are asked to load the root directory '/'.
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, none, _rootKey));
  }
  auto parent = LoadDirBlob(path.parent_path());
  auto entry = parent->GetChild(path.filename().native());

  if (entry.type == fspp::Dir::EntryType::DIR) {
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, std::move(parent), entry.key));
  } else if (entry.type == fspp::Dir::EntryType::FILE) {
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryFile>(this, std::move(parent), entry.key));
  } else if (entry.type == fspp::Dir::EntryType::SYMLINK) {
	return optional<unique_ref<fspp::Node>>(make_unique_ref<CrySymlink>(this, std::move(parent), entry.key));
  } else {
    ASSERT(false, "Unknown entry type");
  }
}

unique_ref<DirBlobRef> CryDevice::LoadDirBlob(const bf::path &path) {
  auto blob = LoadBlob(path);
  auto dir = dynamic_pointer_move<DirBlobRef>(blob);
  if (dir == none) {
    throw FuseErrnoException(ENOTDIR); // Loaded blob is not a directory
  }
  return std::move(*dir);
}

unique_ref<FsBlobRef> CryDevice::LoadBlob(const bf::path &path) {
  auto currentBlob = _fsBlobStore->load(_rootKey);
  ASSERT(currentBlob != none, "rootDir not found");

  for (const bf::path &component : path.relative_path()) {
    auto currentDir = dynamic_pointer_move<DirBlobRef>(*currentBlob);
    if (currentDir == none) {
      throw FuseErrnoException(ENOTDIR); // Path component is not a dir
    }

    Key childKey = (*currentDir)->GetChild(component.c_str()).key;
    currentBlob = _fsBlobStore->load(childKey);
    if (currentBlob == none) {
      throw FuseErrnoException(ENOENT); // Blob for directory entry not found
    }
  }

  return std::move(*currentBlob);

  //TODO Running the python script, waiting for "Create files in sequential order...", then going into dir ~/tmp/cryfs-mount-.../Bonnie.../ and calling "ls"
  //     crashes cryfs with a sigsegv.
  //     Possible reason: Many parallel changes to a directory blob are a race condition. Need something like ParallelAccessStore!
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
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
  ASSERT(blob != none, "Blob not found");
  return std::move(*blob);
}

void CryDevice::RemoveBlob(const blockstore::Key &key) {
  auto blob = _fsBlobStore->load(key);
  ASSERT(blob != none, "Blob not found");
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

}
