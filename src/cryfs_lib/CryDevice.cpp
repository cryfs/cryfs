#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"

#include "fspp/fuse/FuseErrnoException.h"

using std::unique_ptr;

using std::unique_ptr;
using std::make_unique;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blobstore::BlobStore;

namespace cryfs {

CryDevice::CryDevice(unique_ptr<BlobStore> blobStore)
: _blobStore(std::move(blobStore)) {
}

CryDevice::~CryDevice() {
}

unique_ptr<fspp::Node> CryDevice::Load(const bf::path &path) {
  throw FuseErrnoException(ENOTSUP);
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

}
