#include "FuseFTruncateTest.h"

void FuseFTruncateTest::FTruncateFile(const char *filename, off_t size) {
  int error = FTruncateFileReturnError(filename, size);
  EXPECT_EQ(0, error);
}

int FuseFTruncateTest::FTruncateFileReturnError(const char *filename, off_t size) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  int retval = ::ftruncate(fd, size);
  if (0 == retval) {
    return 0;
  } else {
    return errno;
  }
}

int FuseFTruncateTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDWR);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
