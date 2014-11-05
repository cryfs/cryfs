#include <cryfs_lib/CryErrnoException.h>

#include <cstring>
#include <cassert>
#include <string>

using std::string;
using std::runtime_error;

namespace cryfs {

CryErrnoException::CryErrnoException(int errno_)
  :runtime_error(strerror(errno_)), _errno(errno_) {
  assert(_errno != 0);
}

CryErrnoException::~CryErrnoException() {
}

int CryErrnoException::getErrno() const {
  return _errno;
}

} /* namespace cryfs */
