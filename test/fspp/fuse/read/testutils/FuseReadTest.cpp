#include "FuseReadTest.h"

using cpputils::make_unique_ref;
using cpputils::unique_ref;

void FuseReadTest::ReadFile(const char *filename, void *buf, size_t count, off_t offset) {
  auto retval = ReadFileReturnError(filename, buf, count, offset);
  EXPECT_EQ(0, retval.error);
  EXPECT_EQ(count, retval.read_bytes);
}

FuseReadTest::ReadError FuseReadTest::ReadFileReturnError(const char *filename, void *buf, size_t count, off_t offset) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);

  ReadError result{};
  errno = 0;
  result.read_bytes = ::pread(fd->fd(), buf, count, offset);
  result.error = errno;
  return result;
}

unique_ref<OpenFileHandle> FuseReadTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.c_str(), O_RDONLY);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
