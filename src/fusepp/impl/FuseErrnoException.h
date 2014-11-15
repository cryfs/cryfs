#pragma once
#ifndef FSPP_FUSE_FUSEERRNOEXCEPTION_H_
#define FSPP_FUSE_FUSEERRNOEXCEPTION_H_

#include <stdexcept>
#include <errno.h>

namespace fspp {

class FuseErrnoException: public std::runtime_error {
public:
  FuseErrnoException(int errno_);
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

} /* namespace fspp */

#endif /* FSPP_FUSE_FUSEERRNOEXCEPTION_H_ */
