#include "FuseFdatasyncTest.h"

void FuseFdatasyncTest::FdatasyncFile(const char *filename) {
  int error = FdatasyncFileReturnError(filename);
  EXPECT_EQ(0, error);
}

int FuseFdatasyncTest::FdatasyncFileReturnError(const char *filename) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  int retval = ::fdatasync(fd);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

int FuseFdatasyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDWR);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
