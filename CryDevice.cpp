#include <messmer/cryfs/impl/DirBlob.h>
#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "messmer/blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "messmer/blobstore/implementations/onblocks/BlobOnBlocks.h"

using std::unique_ptr;

using std::unique_ptr;
using std::make_unique;
using std::string;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blockstore::BlockStore;
using blockstore::Key;
using blobstore::onblocks::BlobStoreOnBlocks;
using blobstore::onblocks::BlobOnBlocks;

namespace cryfs {

constexpr uint32_t CryDevice::BLOCKSIZE_BYTES;

CryDevice::CryDevice(unique_ptr<CryConfig> config, unique_ptr<BlockStore> blockStore)
: _blobStore(make_unique<BlobStoreOnBlocks>(std::move(blockStore), BLOCKSIZE_BYTES)), _rootKey(GetOrCreateRootKey(config.get())) {
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
  DirBlob rootDir(std::move(rootBlob));
  rootDir.InitializeEmptyDir();
  return rootBlobKey;
}

CryDevice::~CryDevice() {
}

unique_ptr<fspp::Node> CryDevice::Load(const bf::path &path) {
  printf("Loading %s\n", path.c_str());
  assert(path.is_absolute());

  auto currentBlob = _blobStore->load(_rootKey);

  for (const bf::path &component : path.relative_path()) {
    if (!DirBlob::IsDir(*currentBlob)) {
      throw FuseErrnoException(ENOTDIR);
    }
    unique_ptr<DirBlob> currentDir = make_unique<DirBlob>(std::move(currentBlob));

    Key childKey = currentDir->GetBlobKeyForName(component.c_str());
    currentBlob = _blobStore->load(childKey);
  }
  if (DirBlob::IsDir(*currentBlob)) {
    return make_unique<CryDir>(this, std::move(make_unique<DirBlob>(std::move(currentBlob))));
  } else if (FileBlob::IsFile(*currentBlob)) {
    return make_unique<CryFile>(std::move(make_unique<FileBlob>(std::move(currentBlob))));
  } else {
    throw FuseErrnoException(EIO);
  }
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<blobstore::Blob> CryDevice::CreateBlob() {
  return _blobStore->create();
}

}
