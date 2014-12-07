#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"

#include "fspp/fuse/FuseErrnoException.h"
#include "impl/DirBlob.h"

using std::unique_ptr;

using std::unique_ptr;
using std::make_unique;
using std::string;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blobstore::BlobStore;

namespace cryfs {

CryDevice::CryDevice(unique_ptr<CryConfig> config, unique_ptr<BlobStore> blobStore)
: _blob_store(std::move(blobStore)), _root_key(config->RootBlob()) {
  if (_root_key == "") {
    _root_key = CreateRootBlobAndReturnKey();
    config->SetRootBlob(_root_key);
  }
}

string CryDevice::CreateRootBlobAndReturnKey() {
  auto rootBlob = _blob_store->create(DIR_BLOBSIZE);
  DirBlob rootDir(std::move(rootBlob.blob));
  rootDir.InitializeEmptyDir();
  return rootBlob.key;
}

CryDevice::~CryDevice() {
}

unique_ptr<fspp::Node> CryDevice::Load(const bf::path &path) {
  printf("Loading %s\n", path.c_str());
  unique_ptr<DirBlob> currentDir = make_unique<DirBlob>(_blob_store->load(_root_key));
  assert(path.is_absolute());
  for (const bf::path &component : path.relative_path()) {
    printf("Component: %s\n", component.c_str());
    string childKey = currentDir->GetBlobKeyForName(component.c_str());
    currentDir = make_unique<DirBlob>(_blob_store->load(childKey));
  }
  return make_unique<CryDir>(this, std::move(currentDir));
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

blobstore::BlobWithKey CryDevice::CreateBlob(size_t size) {
  return _blob_store->create(size);
}

}
