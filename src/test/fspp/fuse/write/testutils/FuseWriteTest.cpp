#include "FuseWriteTest.h"

void FuseWriteTest::WriteFile(const char *filename, const void *buf, size_t count, off_t offset) {
  size_t retval = WriteFileAllowError(filename, buf, count, offset);
  EXPECT_EQ(count, retval);
}

size_t FuseWriteTest::WriteFileAllowError(const char *filename, const void *buf, size_t count, off_t offset) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  return ::pwrite(fd, buf, count, offset);
}

int FuseWriteTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_WRONLY);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
