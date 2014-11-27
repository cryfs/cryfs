#include "FuseReadTest.h"

void FuseReadTest::ReadFile(const char *filename, void *buf, size_t count, off_t offset) {
  size_t retval = ReadFileAllowError(filename, buf, count, offset);
  EXPECT_EQ(count, retval);
}

size_t FuseReadTest::ReadFileAllowError(const char *filename, void *buf, size_t count, off_t offset) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);
  return ::pread(fd, buf, count, offset);
}

int FuseReadTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDONLY);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
