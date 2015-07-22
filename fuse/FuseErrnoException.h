#pragma once
#ifndef FSPP_FUSE_FUSEERRNOEXCEPTION_H_
#define FSPP_FUSE_FUSEERRNOEXCEPTION_H_

#include <stdexcept>
#include <errno.h>
#include <messmer/cpp-utils/assert/assert.h>

namespace fspp {
namespace fuse{

class FuseErrnoException: public std::runtime_error {
public:
  explicit FuseErrnoException(int errno_);
  virtual ~FuseErrnoException();

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

inline FuseErrnoException::~FuseErrnoException() {
}

inline int FuseErrnoException::getErrno() const {
  return _errno;
}

}
}

#endif /* FSPP_FUSE_FUSEERRNOEXCEPTION_H_ */
