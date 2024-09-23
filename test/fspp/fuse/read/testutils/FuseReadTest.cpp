#include "FuseReadTest.h"
#include "/home/heinzi/projects/cryfs/test/fspp/testutils/OpenFileHandle.h"
#include "fspp/fs_interface/Types.h"
#include <cerrno>
#include <fcntl.h>
#include <gtest/gtest.h>
#include <unistd.h>

using cpputils::make_unique_ref;
using cpputils::unique_ref;

void FuseReadTest::ReadFile(const char *filename, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  auto retval = ReadFileReturnError(filename, buf, count, offset);
  EXPECT_EQ(0, retval.error);
  EXPECT_EQ(count, retval.read_bytes);
}

FuseReadTest::ReadError FuseReadTest::ReadFileReturnError(const char *filename, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);

  ReadError result{};
  errno = 0;
  result.read_bytes = fspp::num_bytes_t(::pread(fd->fd(), buf, count.value(), offset.value()));
  result.error = errno;
  return result;
}

unique_ref<OpenFileHandle> FuseReadTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDONLY);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
