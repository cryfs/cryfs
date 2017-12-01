#include "testutils/FuseFlushTest.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::StrEq;
using ::testing::Eq;
using ::testing::Return;
using ::testing::Throw;
using ::testing::Values;
using ::testing::_;

using fspp::fuse::FuseErrnoException;

class FuseFlushErrorTest: public FuseFlushTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseFlushErrorTest, FuseFlushErrorTest, Values(EBADF, EINTR, EIO));

TEST_P(FuseFlushErrorTest, ReturnErrorFromFlush) {
  ReturnIsFileOnLstat(FILENAME);

  EXPECT_CALL(fsimpl, openFile(StrEq(FILENAME), _)).WillOnce(Return(GetParam()));
  EXPECT_CALL(fsimpl, flush(Eq(GetParam()))).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  auto fs = TestFS();
  auto fd = OpenFile(fs.get(), FILENAME);

  int close_result = ::close(fd->fd());
  EXPECT_EQ(GetParam(), errno);
  EXPECT_EQ(-1, close_result);
  fd->release(); // don't close it again
}
