#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "testutils/FuseOpenTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;

class FuseOpenFileDescriptorTest: public FuseOpenTest, public WithParamInterface<int> {
public:
  void OpenAndReadFile(const char *filename) {
    auto fs = TestFS();

    int fd = OpenFile(fs.get(), filename);
    ReadFile(fd);
  }

private:
  int OpenFile(const TempTestFS *fs, const char *filename) {
    auto realpath = fs->mountDir() / filename;
    int fd = ::open(realpath.c_str(), O_RDONLY);
    EXPECT_GE(fd, 0) << "Opening file failed";
    return fd;
  }
  void ReadFile(int fd) {
    int retval = ::read(fd, nullptr, 0);
    EXPECT_EQ(0, retval) << "Reading file failed";
  }
};
INSTANTIATE_TEST_CASE_P(FuseOpenFileDescriptorTest, FuseOpenFileDescriptorTest, Values(0, 2, 5, 1000, 1024*1024*1024));

TEST_P(FuseOpenFileDescriptorTest, TestReturnedFileDescriptor) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, openFile(StrEq(FILENAME), _))
    .Times(1).WillOnce(Return(GetParam()));
  EXPECT_CALL(fsimpl, read(GetParam(), _, _, _)).Times(1).WillOnce(Return(0));

  OpenAndReadFile(FILENAME);
}
