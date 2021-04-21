#include "testutils/FuseWriteTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Ne;
using ::testing::Invoke;
using ::testing::Throw;

using namespace fspp::fuse;

class FuseWriteErrorTest: public FuseWriteTest, public WithParamInterface<int> {
public:
  fspp::num_bytes_t FILESIZE = fspp::num_bytes_t(64*1024*1024);
  fspp::num_bytes_t WRITECOUNT = fspp::num_bytes_t(32*1024*1024);

  void SetUp() override {
    //Make the file size big enough that fuse should issue at least two writes
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
  }
};
INSTANTIATE_TEST_SUITE_P(FuseWriteErrorTest, FuseWriteErrorTest, Values(EAGAIN, EBADF, EDESTADDRREQ, EDQUOT, EFAULT, EFBIG, EINTR, EINVAL, EIO, ENOSPC, EPIPE, EOVERFLOW, ESPIPE, ENXIO));


TEST_P(FuseWriteErrorTest, ReturnErrorOnFirstWriteCall) {
  EXPECT_CALL(*fsimpl, write(0, testing::_, testing::_, testing::_))
    .WillRepeatedly(Throw(FuseErrnoException(GetParam())));

  char *buf = new char[WRITECOUNT.value()];
  auto retval = WriteFileReturnError(FILENAME, buf, WRITECOUNT, fspp::num_bytes_t(0));
  EXPECT_EQ(GetParam(), retval.error);
  delete[] buf;
}

TEST_P(FuseWriteErrorTest, ReturnErrorOnSecondWriteCall) {
  // The first write request is from the beginning of the file and works, but the later ones fail.
  // We store the number of bytes the first call could successfully write and check later that our
  // write syscall returns exactly this number of bytes
  fspp::num_bytes_t successfullyWrittenBytes = fspp::num_bytes_t(-1);
  EXPECT_CALL(*fsimpl, write(0, testing::_, testing::_, Eq(fspp::num_bytes_t(0))))
    .Times(1)
    .WillOnce(Invoke([&successfullyWrittenBytes](int, const void*, fspp::num_bytes_t count, fspp::num_bytes_t) {
      // Store the number of successfully written bytes
      successfullyWrittenBytes = count;
    }));
  EXPECT_CALL(*fsimpl, write(0, testing::_, testing::_, Ne(fspp::num_bytes_t(0))))
    .WillRepeatedly(Throw(FuseErrnoException(GetParam())));

  char *buf = new char[WRITECOUNT.value()];
  auto retval = WriteFileReturnError(FILENAME, buf, WRITECOUNT, fspp::num_bytes_t(0));
  EXPECT_EQ(0, retval.error);
  EXPECT_EQ(successfullyWrittenBytes, retval.written_bytes); // Check that we're getting the number of successfully written bytes (the first write call) returned
  delete[] buf;
}

