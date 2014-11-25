#include "FuseFstatTest.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Return;

int FuseFstatTest::CreateFile(const TempTestFS *fs, const std::string &filename) {
  int fd = CreateFileAllowErrors(fs, filename);
  EXPECT_GE(fd, 0) << "Opening file failed";
  return fd;
}

int FuseFstatTest::CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename) {
  auto real_path = fs->mountDir() / filename;
  return ::open(real_path.c_str(), O_RDWR | O_CREAT);
}

void FuseFstatTest::OnCreateAndOpenReturnFileDescriptor(const char *filename, int descriptor) {
  EXPECT_CALL(fsimpl, createAndOpenFile(StrEq(filename), _)).Times(1).WillOnce(Return(descriptor));
}
