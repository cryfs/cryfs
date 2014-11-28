#include "FuseFsyncTest.h"

void FuseFsyncTest::FsyncFile(const char *filename) {
  int retval = FsyncFileAllowError(filename);
  EXPECT_EQ(0, retval);
}

int FuseFsyncTest::FsyncFileAllowError(const char *filename) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  return ::fsync(fd);
}

int FuseFsyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDWR);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
