#include "testutils/FuseCreateAndOpenTest.h"

using ::testing::Eq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class FuseCreateAndOpenFileDescriptorTest: public FuseCreateAndOpenTest, public WithParamInterface<int> {
public:
  void CreateAndOpenAndReadFile(const char *filename) {
    auto fs = TestFS();

    auto fd = CreateAndOpenFile(fs.get(), filename);
    ReadFile(fd->fd());
  }

private:
  unique_ref<OpenFileHandle> CreateAndOpenFile(const TempTestFS *fs, const char *filename) {
    auto realpath = fs->mountDir() / filename;
    auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDONLY | O_CREAT, S_IRUSR | S_IRGRP | S_IROTH);
    EXPECT_GE(fd->fd(), 0) << "Creating file failed";
    return fd;
  }
  void ReadFile(int fd) {
    uint8_t buf = 0;
    int retval = ::read(fd, &buf, 1);
    EXPECT_EQ(1, retval) << "Reading file failed";
  }
};
INSTANTIATE_TEST_SUITE_P(FuseCreateAndOpenFileDescriptorTest, FuseCreateAndOpenFileDescriptorTest, Values(0, 2, 5, 1000, 1024*1024*1024));

TEST_P(FuseCreateAndOpenFileDescriptorTest, TestReturnedFileDescriptor) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(FILENAME), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Return(GetParam()));
  EXPECT_CALL(*fsimpl, read(GetParam(), testing::_, testing::_, testing::_)).Times(1).WillOnce(Return(fspp::num_bytes_t(1)));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstatWithSize(GetParam(), fspp::num_bytes_t(1));

  CreateAndOpenAndReadFile(FILENAME);
}
