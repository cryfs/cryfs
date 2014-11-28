#include "testutils/FuseStatfsTest.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Throw;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

using fspp::fuse::FuseErrnoException;

class FuseStatfsErrorTest: public FuseStatfsTest, public WithParamInterface<int> {
public:
};
INSTANTIATE_TEST_CASE_P(FuseStatfsErrorTest, FuseStatfsErrorTest, Values(EACCES, EBADF, EFAULT, EINTR, EIO, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOSYS, ENOTDIR, EOVERFLOW));

TEST_F(FuseStatfsErrorTest, ReturnNoError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, statfs(StrEq(FILENAME), _)).Times(1).WillOnce(Return());
  errno = 0;
  int retval = StatfsAllowErrors(FILENAME);
  EXPECT_EQ(errno, 0);
  EXPECT_EQ(retval, 0);
}

TEST_P(FuseStatfsErrorTest, ReturnError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, statfs(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));
  int retval = StatfsAllowErrors(FILENAME);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(GetParam(), errno);
}
