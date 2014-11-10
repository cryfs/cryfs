#include <cryfs_lib/CryOpenFile.h>

#include <sys/types.h>
#include <fcntl.h>

#include "CryErrnoException.h"

using namespace cryfs;

CryOpenFile::CryOpenFile(const bf::path &path, int flags)
  :_descriptor(::open(path.c_str(), flags)) {
  CHECK_RETVAL(_descriptor);
}

CryOpenFile::~CryOpenFile() {
  int retval = close(_descriptor);
  CHECK_RETVAL(retval);
}

void CryOpenFile::stat(struct ::stat *result) const {
  int retval = ::fstat(_descriptor, result);
  CHECK_RETVAL(retval);
}

void CryOpenFile::truncate(off_t size) const {
  int retval = ::ftruncate(_descriptor, size);
  CHECK_RETVAL(retval);
}

void CryOpenFile::read(void *buf, size_t count, off_t offset) {
  int retval = ::pread(_descriptor, buf, count, offset);
  CHECK_RETVAL(retval);
}

void CryOpenFile::write(const void *buf, size_t count, off_t offset) {
  int retval = ::pwrite(_descriptor, buf, count, offset);
  CHECK_RETVAL(retval);
}

void CryOpenFile::fsync() {
  int retval = ::fsync(_descriptor);
  CHECK_RETVAL(retval);
}

void CryOpenFile::fdatasync() {
  int retval = ::fdatasync(_descriptor);
  CHECK_RETVAL(retval);
}
