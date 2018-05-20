#include "FuseFlushTest.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;

void FuseFlushTest::OpenAndCloseFile(const std::string &filename) {
  auto fs = TestFS();
  auto fd = OpenFile(fs.get(), filename);
  CloseFile(fd->fd());
  fd->release(); // don't try to close it again
}

unique_ref<OpenFileHandle> FuseFlushTest::OpenFile(const TempTestFS *fs, const std::string &filename) {
  auto real_path = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(real_path.string().c_str(), O_RDONLY);
  EXPECT_GE(fd->fd(), 0) << "Opening file failed";
  return fd;
}

void FuseFlushTest::CloseFile(int fd) {
  int retval = ::close(fd);
  EXPECT_EQ(0, retval);
}
