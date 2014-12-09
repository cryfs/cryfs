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
  assert(path.is_absolute());

  auto current_blob = _blob_store->load(_root_key);

  for (const bf::path &component : path.relative_path()) {
    if (!DirBlob::IsDir(*current_blob)) {
      throw FuseErrnoException(ENOTDIR);
    }
    unique_ptr<DirBlob> currentDir = make_unique<DirBlob>(std::move(current_blob));

    string childKey = currentDir->GetBlobKeyForName(component.c_str());
    current_blob = _blob_store->load(childKey);
  }
  if (DirBlob::IsDir(*current_blob)) {
    return make_unique<CryDir>(this, std::move(make_unique<DirBlob>(std::move(current_blob))));
  } else if (FileBlob::IsFile(*current_blob)) {
    return make_unique<CryFile>(std::move(make_unique<FileBlob>(std::move(current_blob))));
  } else {
    throw FuseErrnoException(EIO);
  }
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

blobstore::BlobWithKey CryDevice::CreateBlob(size_t size) {
  return _blob_store->create(size);
}

}
