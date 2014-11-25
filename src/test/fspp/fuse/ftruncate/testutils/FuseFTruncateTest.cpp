#include "FuseFTruncateTest.h"

void FuseFTruncateTest::FTruncateFile(const char *filename, off_t size) {
  int retval = FTruncateFileAllowError(filename, size);
  EXPECT_EQ(0, retval);
}

int FuseFTruncateTest::FTruncateFileAllowError(const char *filename, off_t size) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  return ::ftruncate(fd, size);
}

int FuseFTruncateTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDWR);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
