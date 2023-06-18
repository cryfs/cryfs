#include "FuseFstatTest.h"

using ::testing::Eq;
using ::testing::Return;
using cpputils::unique_ref;
using cpputils::make_unique_ref;


unique_ref<OpenFileHandle> FuseFstatTest::CreateFile(const TempTestFS *fs, const std::string &filename) {
  auto fd = CreateFileAllowErrors(fs, filename);
  EXPECT_GE(fd->fd(), 0) << "Opening file failed";
  return fd;
}

int FuseFstatTest::CreateFileReturnError(const TempTestFS *fs, const std::string &filename) {
  auto fd = CreateFileAllowErrors(fs, filename);
  return fd->errorcode();
}

unique_ref<OpenFileHandle> FuseFstatTest::CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename) {
  auto real_path = fs->mountDir() / filename;
  auto fd = make_unique_ref<OpenFileHandle>(real_path.string().c_str(), O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);
  return fd;
}

void FuseFstatTest::OnCreateAndOpenReturnFileDescriptor(const char *filename, int descriptor) {
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(filename), testing::_, testing::_, testing::_)).Times(1).WillOnce(Return(descriptor));
}
