#include "../cryfs_lib/CryDevice.h"

#include <memory>

#include "CryDir.h"
#include "CryFile.h"
#include "CryErrnoException.h"

using namespace cryfs;

using std::unique_ptr;
using std::make_unique;

CryDevice::CryDevice(const bf::path &rootdir)
  :_rootdir(rootdir) {
}

CryDevice::~CryDevice() {
}

unique_ptr<CryNode> CryDevice::LoadFromPath(const bf::path &path) {
  auto real_path = RootDir() / path;
  if(bf::is_directory(real_path)) {
    return make_unique<CryDir>(this, path);
  } else if(bf::is_regular_file(real_path)) {
    return make_unique<CryFile>(this, path);
  }

  throw CryErrnoException(ENOENT);
}
