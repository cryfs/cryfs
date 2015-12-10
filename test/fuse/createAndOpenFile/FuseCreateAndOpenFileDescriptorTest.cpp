#include "testutils/FuseCreateAndOpenTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;

class FuseCreateAndOpenFileDescriptorTest: public FuseCreateAndOpenTest, public WithParamInterface<int> {
public:
  void CreateAndOpenAndReadFile(const char *filename) {
    auto fs = TestFS();

    int fd = CreateAndOpenFile(fs.get(), filename);
    ReadFile(fd);
  }

private:
  int CreateAndOpenFile(const TempTestFS *fs, const char *filename) {
    auto realpath = fs->mountDir() / filename;
    int fd = ::open(realpath.c_str(), O_RDONLY | O_CREAT, S_IRUSR | S_IRGRP | S_IROTH);
    EXPECT_GE(fd, 0) << "Creating file failed";
    return fd;
  }
  void ReadFile(int fd) {
    uint8_t buf;
    int retval = ::read(fd, &buf, 1);
    EXPECT_EQ(1, retval) << "Reading file failed";
  }
};
INSTANTIATE_TEST_CASE_P(FuseCreateAndOpenFileDescriptorTest, FuseCreateAndOpenFileDescriptorTest, Values(0, 2, 5, 1000, 1024*1024*1024));

TEST_P(FuseCreateAndOpenFileDescriptorTest, TestReturnedFileDescriptor) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, createAndOpenFile(StrEq(FILENAME), _, _, _))
    .Times(1).WillOnce(Return(GetParam()));
  EXPECT_CALL(fsimpl, read(GetParam(), _, _, _)).Times(1).WillOnce(Return(1));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstatWithSize(GetParam(), 1);

  CreateAndOpenAndReadFile(FILENAME);
}
