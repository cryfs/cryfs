#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of a exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryOpenFile::CryOpenFile() {
  throw FuseErrnoException(ENOTSUP);
}

CryOpenFile::~CryOpenFile() {
  //TODO
}

void CryOpenFile::flush() {
  throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::stat(struct ::stat *result) const {
  throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::truncate(off_t size) const {
  throw FuseErrnoException(ENOTSUP);
}

int CryOpenFile::read(void *buf, size_t count, off_t offset) {
  throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::write(const void *buf, size_t count, off_t offset) {
  throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::fsync() {
  throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::fdatasync() {
  throw FuseErrnoException(ENOTSUP);
}

}
