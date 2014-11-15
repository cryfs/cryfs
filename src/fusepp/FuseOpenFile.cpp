#include <sys/types.h>
#include <fcntl.h>
#include <fusepp/FuseDevice.h>
#include <fusepp/FuseErrnoException.h>
#include <fusepp/FuseOpenFile.h>


using namespace fusepp;

FuseOpenFile::FuseOpenFile(const FuseDevice *device, const bf::path &path, int flags)
  :_descriptor(::open((device->RootDir() / path).c_str(), flags)) {
  CHECK_RETVAL(_descriptor);
}

FuseOpenFile::~FuseOpenFile() {
  int retval = close(_descriptor);
  CHECK_RETVAL(retval);
}

void FuseOpenFile::stat(struct ::stat *result) const {
  int retval = ::fstat(_descriptor, result);
  CHECK_RETVAL(retval);
}

void FuseOpenFile::truncate(off_t size) const {
  int retval = ::ftruncate(_descriptor, size);
  CHECK_RETVAL(retval);
}

int FuseOpenFile::read(void *buf, size_t count, off_t offset) {
  //printf("Reading from real descriptor %d (%d, %d)\n", _descriptor, offset, count);
  //fflush(stdout);
  int retval = ::pread(_descriptor, buf, count, offset);
  CHECK_RETVAL(retval);
  //printf("retval: %d, count: %d\n", retval, count);
  //fflush(stdout);
  assert(static_cast<unsigned int>(retval) <= count);
  return retval;
}

void FuseOpenFile::write(const void *buf, size_t count, off_t offset) {
  int retval = ::pwrite(_descriptor, buf, count, offset);
  CHECK_RETVAL(retval);
  assert(static_cast<unsigned int>(retval) == count);
}

void FuseOpenFile::fsync() {
  int retval = ::fsync(_descriptor);
  CHECK_RETVAL(retval);
}

void FuseOpenFile::fdatasync() {
  int retval = ::fdatasync(_descriptor);
  CHECK_RETVAL(retval);
}
