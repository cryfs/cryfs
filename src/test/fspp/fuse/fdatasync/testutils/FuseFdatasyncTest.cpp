#include "FuseFdatasyncTest.h"

void FuseFdatasyncTest::FdatasyncFile(const char *filename) {
  int retval = FdatasyncFileAllowError(filename);
  EXPECT_EQ(0, retval);
}

int FuseFdatasyncTest::FdatasyncFileAllowError(const char *filename) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  return ::fdatasync(fd);
}

int FuseFdatasyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDWR);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
