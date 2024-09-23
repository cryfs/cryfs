#include "FuseFsyncTest.h"
#include "/home/heinzi/projects/cryfs/test/fspp/testutils/OpenFileHandle.h"
#include <cerrno>
#include <fcntl.h>
#include <gtest/gtest.h>
#include <unistd.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseFsyncTest::FsyncFile(const char *filename) {
  const int error = FsyncFileReturnError(filename);
  EXPECT_EQ(0, error);
}

int FuseFsyncTest::FsyncFileReturnError(const char *filename) {
  auto fs = TestFS();

  auto fd = OpenFile(fs.get(), filename);
  const int retval = ::fsync(fd->fd());
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

unique_ref<OpenFileHandle> FuseFsyncTest::OpenFile(const TempTestFS *fs, const char *filename) {
  auto realpath = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDWR);
  EXPECT_GE(fd->fd(), 0) << "Error opening file";
  return fd;
}
