#include "FuseFlushTest.h"

void FuseFlushTest::OpenAndCloseFile(const std::string &filename) {
  auto fs = TestFS();
  int fd = OpenFile(fs.get(), filename);
  CloseFile(fd);
}

int FuseFlushTest::OpenFile(const TempTestFS *fs, const std::string &filename) {
  auto real_path = fs->mountDir() / filename;
  int fd = ::open(real_path.c_str(), O_RDONLY);
  EXPECT_GE(fd, 0) << "Opening file failed";
  return fd;
}

void FuseFlushTest::CloseFile(int fd) {
  int retval = ::close(fd);
  EXPECT_EQ(0, retval);
}
