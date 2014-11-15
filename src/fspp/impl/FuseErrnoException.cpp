#include "FuseErrnoException.h"

#include <cstring>
#include <cassert>
#include <string>

using std::string;
using std::runtime_error;

namespace fspp {

FuseErrnoException::FuseErrnoException(int errno_)
  :runtime_error(strerror(errno_)), _errno(errno_) {
  assert(_errno != 0);
}

FuseErrnoException::~FuseErrnoException() {
}

int FuseErrnoException::getErrno() const {
  return _errno;
}

} /* namespace fspp */
