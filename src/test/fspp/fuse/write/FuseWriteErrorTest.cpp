#include "testutils/FuseWriteTest.h"

#include "fspp/impl/FuseErrnoException.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Ne;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::Throw;

using namespace fspp;

class FuseWriteErrorTest: public FuseWriteTest, public WithParamInterface<int> {
public:
  size_t FILESIZE = 1024*1024*1024;
  size_t WRITECOUNT = 512*1024*1024;

  void SetUp() override {
    //Make the file size big enough that fuse should issue at least two writes
    ReturnIsFileOnLstatWithSize(FILENAME, FILESIZE);
    OnOpenReturnFileDescriptor(FILENAME, 0);
  }
};
INSTANTIATE_TEST_CASE_P(FuseWriteErrorTest, FuseWriteErrorTest, Values(EAGAIN, EBADF, EDESTADDRREQ, EDQUOT, EFAULT, EFBIG, EINTR, EINVAL, EIO, ENOSPC, EPIPE, EOVERFLOW, ESPIPE, ENXIO));


TEST_P(FuseWriteErrorTest, ReturnErrorOnFirstWriteCall) {
  EXPECT_CALL(fsimpl, write(0, _, _, _))
    .WillRepeatedly(Throw(FuseErrnoException(GetParam())));

  char *buf = new char[WRITECOUNT];
  errno = 0;
  int retval = WriteFileAllowError(FILENAME, buf, WRITECOUNT, 0);
  EXPECT_EQ(GetParam(), errno);
  EXPECT_EQ(-1, retval);
  delete[] buf;
}

TEST_P(FuseWriteErrorTest, ReturnErrorOnSecondWriteCall) {
  // The first write request is from the beginning of the file and works, but the later ones fail.
  // We store the number of bytes the first call could successfully write and check later that our
  // write syscall returns exactly this number of bytes
  size_t successfullyWrittenBytes = -1;
  EXPECT_CALL(fsimpl, write(0, _, _, Eq(0)))
    .Times(1)
    .WillOnce(Invoke([&successfullyWrittenBytes](int, const void*, size_t count, off_t) {
      // Store the number of successfully written bytes
      successfullyWrittenBytes = count;
    }));
  EXPECT_CALL(fsimpl, write(0, _, _, Ne(0)))
    .WillRepeatedly(Throw(FuseErrnoException(GetParam())));

  char *buf = new char[WRITECOUNT];
  errno = 0;
  size_t retval = WriteFileAllowError(FILENAME, buf, WRITECOUNT, 0);
  EXPECT_EQ(0, errno);
  EXPECT_EQ(successfullyWrittenBytes, retval); // Check that we're getting the number of successfully written bytes (the first write call) returned
  delete[] buf;
}

