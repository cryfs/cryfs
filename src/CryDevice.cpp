#include <messmer/blockstore/implementations/caching/CachingBlockStore.h>
#include <messmer/blockstore/implementations/encrypted/ciphers/AES256_CFB.h>
#include "impl/DirBlob.h"
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "messmer/blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "messmer/blobstore/implementations/onblocks/BlobOnBlocks.h"
#include "messmer/blockstore/implementations/encrypted/EncryptedBlockStore.h"

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
using boost::optional;
using boost::none;

namespace bf = boost::filesystem;

namespace cryfs {

constexpr uint32_t CryDevice::BLOCKSIZE_BYTES;

CryDevice::CryDevice(unique_ref<CryConfig> config, unique_ref<BlockStore> blockStore)
: _blobStore(make_unique_ref<BlobStoreOnBlocks>(make_unique_ref<CachingBlockStore>(make_unique_ref<EncryptedBlockStore<Cipher>>(std::move(blockStore), GetEncryptionKey(config.get()))), BLOCKSIZE_BYTES)), _rootKey(GetOrCreateRootKey(config.get())) {
}

Key CryDevice::GetOrCreateRootKey(CryConfig *config) {
  string root_key = config->RootBlob();
  if (root_key == "") {
    auto new_key = CreateRootBlobAndReturnKey();
    config->SetRootBlob(new_key.ToString());
    config->save();
    return new_key;
  }

  return Key::FromString(root_key);
}

CryDevice::Cipher::EncryptionKey CryDevice::GetEncryptionKey(CryConfig *config) {
  return Cipher::EncryptionKey::FromString(config->EncryptionKey());
}

Key CryDevice::CreateRootBlobAndReturnKey() {
  auto rootBlob = _blobStore->create();
  Key rootBlobKey = rootBlob->key();
  DirBlob::InitializeEmptyDir(std::move(rootBlob), this);
  return rootBlobKey;
}

CryDevice::~CryDevice() {
}

optional<unique_ref<fspp::Node>> CryDevice::Load(const bf::path &path) {
  ASSERT(path.is_absolute(), "Non absolute path given");

  if (path.parent_path().empty()) {
    //We are asked to load the root directory '/'.
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, none, _rootKey));
  }
  auto parent = LoadDirBlob(path.parent_path());
  if (parent == none) {
    //TODO Return correct fuse error
    return none;
  }
  auto entry = (*parent)->GetChild(path.filename().native());

  if (entry.type == fspp::Dir::EntryType::DIR) {
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryDir>(this, std::move(*parent), entry.key));
  } else if (entry.type == fspp::Dir::EntryType::FILE) {
    return optional<unique_ref<fspp::Node>>(make_unique_ref<CryFile>(this, std::move(*parent), entry.key));
  } else if (entry.type == fspp::Dir::EntryType::SYMLINK) {
	return optional<unique_ref<fspp::Node>>(make_unique_ref<CrySymlink>(this, std::move(*parent), entry.key));
  } else {
    throw FuseErrnoException(EIO);
  }
}

optional<unique_ref<DirBlob>> CryDevice::LoadDirBlob(const bf::path &path) {
  auto currentBlob = _blobStore->load(_rootKey);
  if(currentBlob == none) {
    //TODO Return correct fuse error
    return none;
  }

  for (const bf::path &component : path.relative_path()) {
    //TODO Check whether the next path component is a dir.
    //     Right now, an assertion in DirBlob constructor will fail if it isn't.
    //     But fuse should rather return the correct error code.
    unique_ref<DirBlob> currentDir = make_unique_ref<DirBlob>(std::move(*currentBlob), this);

    Key childKey = currentDir->GetChild(component.c_str()).key;
    currentBlob = _blobStore->load(childKey);
    if(currentBlob == none) {
      //TODO Return correct fuse error
      return none;
    }
  }

  return make_unique_ref<DirBlob>(std::move(*currentBlob), this);
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

unique_ref<blobstore::Blob> CryDevice::CreateBlob() {
  return _blobStore->create();
}

optional<unique_ref<blobstore::Blob>> CryDevice::LoadBlob(const blockstore::Key &key) {
  return _blobStore->load(key);
}

void CryDevice::RemoveBlob(const blockstore::Key &key) {
  auto blob = _blobStore->load(key);
  if(blob) {
    _blobStore->remove(std::move(*blob));
  } else {
    //TODO Log error
  }
}

}
