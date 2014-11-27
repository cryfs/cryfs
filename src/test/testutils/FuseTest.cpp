#include "FuseTest.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Return;

MockFilesystem::MockFilesystem() {}
MockFilesystem::~MockFilesystem() {}

void FuseTest::OnOpenReturnFileDescriptor(const char *filename, int descriptor) {
  EXPECT_CALL(fsimpl, openFile(StrEq(filename), _)).Times(1).WillOnce(Return(descriptor));
}

void FuseTest::ReturnIsFileOnLstat(const bf::path &path) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnIsFile);
}

void FuseTest::ReturnIsFileOnLstatWithSize(const bf::path &path, const size_t size) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnIsFileWithSize(size));
}

void FuseTest::ReturnIsDirOnLstat(const bf::path &path) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnIsDir);
}

void FuseTest::ReturnDoesntExistOnLstat(const bf::path &path) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnDoesntExist);
}

void FuseTest::ReturnIsFileOnFstat(int descriptor) {
  EXPECT_CALL(fsimpl, fstat(descriptor, _)).WillRepeatedly(ReturnIsFileFstat);
}
