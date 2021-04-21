#include "testutils/FuseFlushTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Eq;
using ::testing::Return;
using ::testing::Throw;
using ::testing::Values;

using fspp::fuse::FuseErrnoException;

class FuseFlushErrorTest: public FuseFlushTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFlushErrorTest, FuseFlushErrorTest, Values(
    EBADF,
#if defined(__GLIBC__) || defined(__APPLE__)
    // musl has different handling for EINTR, see https://ewontfix.com/4/
    EINTR,
#endif
    EIO));

TEST_P(FuseFlushErrorTest, ReturnErrorFromFlush) {
  ReturnIsFileOnLstat(FILENAME);

  EXPECT_CALL(*fsimpl, openFile(Eq(FILENAME), testing::_)).WillOnce(Return(GetParam()));
  EXPECT_CALL(*fsimpl, flush(Eq(GetParam()))).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  auto fs = TestFS();
  auto fd = OpenFile(fs.get(), FILENAME);

  int close_result = ::close(fd->fd());
  EXPECT_EQ(GetParam(), errno);
  EXPECT_EQ(-1, close_result);
  fd->release(); // don't close it again
}
