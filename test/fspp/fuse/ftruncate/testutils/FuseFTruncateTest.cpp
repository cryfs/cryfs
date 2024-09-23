#include "FuseFTruncateTest.h"
#include "/home/heinzi/projects/cryfs/test/fspp/testutils/OpenFileHandle.h"
#include "fspp/fs_interface/Types.h"
#include <cerrno>
#include <fcntl.h>
#include <gtest/gtest.h>
#include <unistd.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseFTruncateTest::FTruncateFile(const char *filename, fspp::num_bytes_t size) {
  const int error = FTruncateFileReturnError(filename, size);
  EXPECT_EQ(0, error);
}

int FuseFTruncateTest::FTruncateFileReturnError(const char *filename, fspp::num_bytes_t size) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);
  const int retval = ::ftruncate(fd->fd(), size.value());
  if (0 == retval) {
    return 0;
  } else {
    return errno;
  }
}

unique_ref<OpenFileHandle> FuseFTruncateTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDWR);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
