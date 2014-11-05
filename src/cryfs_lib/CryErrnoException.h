#pragma once
#ifndef CRYFS_LIB_CRYERRNOEXCEPTION_H_
#define CRYFS_LIB_CRYERRNOEXCEPTION_H_

#include <stdexcept>
#include <errno.h>

namespace cryfs {

class CryErrnoException: public std::runtime_error {
public:
  CryErrnoException(int errno_);
  virtual ~CryErrnoException();

  int getErrno() const;
private:
  int _errno;
};

inline void CHECK_RETVAL(int retval) {
  if (retval < 0) {
    throw CryErrnoException(errno);
  }
}

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYERRNOEXCEPTION_H_ */
