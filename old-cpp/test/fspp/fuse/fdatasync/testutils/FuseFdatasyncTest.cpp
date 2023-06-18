#include "FuseFdatasyncTest.h"
#include <fcntl.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseFdatasyncTest::FdatasyncFile(const char *filename) {
  int error = FdatasyncFileReturnError(filename);
  EXPECT_EQ(0, error);
}

int FuseFdatasyncTest::FdatasyncFileReturnError(const char *filename) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);
#ifdef F_FULLFSYNC
  // This is MacOSX, which doesn't know fdatasync
  int retval = fcntl(fd->fd(), F_FULLFSYNC);
#else
  int retval = ::fdatasync(fd->fd());
#endif
  if (retval != -1) {
    return 0;
  } else {
    return errno;
  }
}

unique_ref<OpenFileHandle> FuseFdatasyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDWR);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
