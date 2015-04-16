#include <messmer/blockstore/implementations/caching/CachingBlockStore.h>
#include "impl/DirBlob.h"
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "messmer/blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "messmer/blobstore/implementations/onblocks/BlobOnBlocks.h"
#include "messmer/blockstore/implementations/encrypted/EncryptedBlockStore.h"

using std::unique_ptr;
using std::make_unique;
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

namespace cryfs {

constexpr uint32_t CryDevice::BLOCKSIZE_BYTES;

CryDevice::CryDevice(unique_ptr<CryConfig> config, unique_ptr<BlockStore> blockStore)
: _blobStore(make_unique<BlobStoreOnBlocks>(make_unique<CachingBlockStore>(make_unique<EncryptedBlockStore>(std::move(blockStore), config->EncryptionKey())), BLOCKSIZE_BYTES)), _rootKey(GetOrCreateRootKey(config.get())) {
}

Key CryDevice::GetOrCreateRootKey(CryConfig *config) {
  string root_key = config->RootBlob();
  if (root_key == "") {
    auto key = CreateRootBlobAndReturnKey();
    config->SetRootBlob(key.ToString());
    return key;
  }

  return Key::FromString(root_key);
}

Key CryDevice::CreateRootBlobAndReturnKey() {
  auto rootBlob = _blobStore->create();
  Key rootBlobKey = rootBlob->key();
  DirBlob::InitializeEmptyDir(std::move(rootBlob));
  return rootBlobKey;
}

CryDevice::~CryDevice() {
}

unique_ptr<fspp::Node> CryDevice::Load(const bf::path &path) {
  printf("Loading %s\n", path.c_str());
  assert(path.is_absolute());

  if (path.parent_path().empty()) {
    //We are asked to load the root directory '/'.
    return make_unique<CryDir>(this, nullptr, _rootKey);
  }
  auto parent = LoadDirBlob(path.parent_path());
  auto entry = parent->GetChild(path.filename().native());

  if (entry.type == fspp::Dir::EntryType::DIR) {
    return make_unique<CryDir>(this, std::move(parent), entry.key);
  } else if (entry.type == fspp::Dir::EntryType::FILE) {
    return make_unique<CryFile>(this, std::move(parent), entry.key);
  } else {
    throw FuseErrnoException(EIO);
  }
}

unique_ptr<DirBlob> CryDevice::LoadDirBlob(const bf::path &path) {
  auto currentBlob = _blobStore->load(_rootKey);

  for (const bf::path &component : path.relative_path()) {
    //TODO Check whether the next path component is a dir.
    //     Right now, an assertion in DirBlob constructor will fail if it isn't.
    //     But fuse should rather return the correct error code.
    unique_ptr<DirBlob> currentDir = make_unique<DirBlob>(std::move(currentBlob));

    Key childKey = currentDir->GetChild(component.c_str()).key;
    currentBlob = _blobStore->load(childKey);
  }

  return make_unique<DirBlob>(std::move(currentBlob));
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<blobstore::Blob> CryDevice::CreateBlob() {
  return _blobStore->create();
}

unique_ptr<blobstore::Blob> CryDevice::LoadBlob(const blockstore::Key &key) {
  return _blobStore->load(key);
}

void CryDevice::RemoveBlob(const blockstore::Key &key) {
  _blobStore->remove(_blobStore->load(key));
}

}
