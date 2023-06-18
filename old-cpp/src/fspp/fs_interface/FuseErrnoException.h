#pragma once
#ifndef MESSMER_FSPP_FUSE_FUSEERRNOEXCEPTION_H_
#define MESSMER_FSPP_FUSE_FUSEERRNOEXCEPTION_H_

#include <stdexcept>
#include <errno.h>
#include <cpp-utils/assert/assert.h>

// TODO Need a portable way to report errors

namespace fspp {
namespace fuse{

class FuseErrnoException final: public std::runtime_error {
public:
  explicit FuseErrnoException(int errno_);

  int getErrno() const;
private:
  int _errno;
};

inline void CHECK_RETVAL(int retval) {
  if (retval < 0) {
    throw FuseErrnoException(errno);
  }
}

inline FuseErrnoException::FuseErrnoException(int errno_)
  :runtime_error(strerror(errno_)), _errno(errno_) {
  ASSERT(_errno != 0, "Errno shouldn't be zero");
}

inline int FuseErrnoException::getErrno() const {
  return _errno;
}

}
}

#endif
