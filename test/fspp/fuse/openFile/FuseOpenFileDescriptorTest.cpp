#include "testutils/FuseOpenTest.h"

using ::testing::Eq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class FuseOpenFileDescriptorTest: public FuseOpenTest, public WithParamInterface<int> {
public:
  void OpenAndReadFile(const char *filename) {
    auto fs = TestFS();

    auto fd = OpenFile(fs.get(), filename);
    ReadFile(fd->fd());
  }

private:
  unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs, const char *filename) {
    auto realpath = fs->mountDir() / filename;
    auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDONLY);
    EXPECT_GE(fd->fd(), 0) << "Opening file failed";
    return fd;
  }
  void ReadFile(int fd) {
    uint8_t buf = 0;
    int retval = ::read(fd, &buf, 1);
    EXPECT_EQ(1, retval) << "Reading file failed";
  }
};
INSTANTIATE_TEST_SUITE_P(FuseOpenFileDescriptorTest, FuseOpenFileDescriptorTest, Values(0, 2, 5, 1000, 1024*1024*1024));

TEST_P(FuseOpenFileDescriptorTest, TestReturnedFileDescriptor) {
  ReturnIsFileOnLstatWithSize(FILENAME, fspp::num_bytes_t(1));
  EXPECT_CALL(*fsimpl, openFile(Eq(FILENAME), testing::_))
    .Times(1).WillOnce(Return(GetParam()));
  EXPECT_CALL(*fsimpl, read(GetParam(), testing::_, testing::_, testing::_)).Times(1).WillOnce(Return(fspp::num_bytes_t(1)));

  OpenAndReadFile(FILENAME);
}

