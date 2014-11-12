#include <cryfs_lib/CryOpenDir.h>

#include "CryDevice.h"
#include "CryErrnoException.h"

using std::unique_ptr;
using std::make_unique;
using std::vector;
using std::string;

namespace cryfs {

CryOpenDir::CryOpenDir(const CryDevice *device, const bf::path &path)
  :_dir(::opendir((device->RootDir() / path).c_str())) {
  if (_dir == nullptr) {
    throw CryErrnoException(errno);
  }
}

CryOpenDir::~CryOpenDir() {
  int retval = ::closedir(_dir);
  CHECK_RETVAL(retval);
}

unique_ptr<vector<string>> CryOpenDir::readdir() const {
  ::rewinddir(_dir);

  auto result = make_unique<vector<string>>();

  struct dirent *entry = ::readdir(_dir);
  while(entry != nullptr) {
    result->push_back(entry->d_name);
    entry = ::readdir(_dir);
  }
  //On error, ::readdir returns nullptr and sets errno.
  if (errno != 0) {
    throw CryErrnoException(errno);
  }
  return result;
}

} /* namespace cryfs */
