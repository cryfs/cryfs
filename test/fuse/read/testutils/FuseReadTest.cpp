#include "FuseReadTest.h"

void FuseReadTest::ReadFile(const char *filename, void *buf, size_t count, off_t offset) {
  auto retval = ReadFileReturnError(filename, buf, count, offset);
  EXPECT_EQ(0, retval.error);
  EXPECT_EQ(count, retval.read_bytes);
}

FuseReadTest::ReadError FuseReadTest::ReadFileReturnError(const char *filename, void *buf, size_t count, off_t offset) {
  auto fs = TestFS();

  int fd = OpenFile(fs.get(), filename);

  ReadError result;
  errno = 0;
  result.read_bytes = ::pread(fd, buf, count, offset);
  result.error = errno;
  return result;
}

int FuseReadTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  int fd = ::open(realpath.c_str(), O_RDONLY);
  EXPECT_GE(fd, 0) << "Error opening file";
  return fd;
}
