#include "FuseWriteTest.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseWriteTest::WriteFile(const char *filename, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  auto retval = WriteFileReturnError(filename, buf, count, offset);
  EXPECT_EQ(0, retval.error);
  EXPECT_EQ(count, retval.written_bytes);
}

FuseWriteTest::WriteError FuseWriteTest::WriteFileReturnError(const char *filename, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);

  WriteError result{};
  errno = 0;
  result.written_bytes = fspp::num_bytes_t(::pwrite(fd->fd(), buf, count.value(), offset.value()));
  result.error = errno;
  return result;
}

unique_ref<OpenFileHandle> FuseWriteTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_WRONLY);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
