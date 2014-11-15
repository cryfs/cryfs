#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"

#include "fusepp/impl/FuseErrnoException.h"

using std::unique_ptr;

using std::unique_ptr;
using std::make_unique;

//TODO Get rid of this in favor of exception hierarchy
using fusepp::CHECK_RETVAL;

namespace cryfs {

CryDevice::CryDevice(const bf::path &root_path): _root_path(root_path) {
}

CryDevice::~CryDevice() {
}

unique_ptr<fusepp::FuseNode> CryDevice::Load(const bf::path &path) {
  auto real_path = RootDir() / path;
  if(bf::is_directory(real_path)) {
    return make_unique<CryDir>(this, path);
  } else if(bf::is_regular_file(real_path)) {
    return make_unique<CryFile>(this, path);
  }

  throw fusepp::FuseErrnoException(ENOENT);
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  auto real_path = RootDir() / path;
  int retval = ::statvfs(real_path.c_str(), fsstat);
  CHECK_RETVAL(retval);
}

} /* namespace cryfs */
