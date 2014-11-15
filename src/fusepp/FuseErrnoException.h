#pragma once
#ifndef FUSEPP_FUSEERRNOEXCEPTION_H_
#define FUSEPP_FUSEERRNOEXCEPTION_H_

#include <stdexcept>
#include <errno.h>

namespace fusepp {

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

} /* namespace fusepp */

#endif /* FUSEPP_FUSEERRNOEXCEPTION_H_ */
