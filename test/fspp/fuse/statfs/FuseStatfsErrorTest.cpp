#include "testutils/FuseStatfsTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

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
  EXPECT_CALL(fsimpl, statfs(_)).Times(1).WillOnce(Return());
  int error = StatfsReturnError(FILENAME);
  EXPECT_EQ(0, error);
}

TEST_P(FuseStatfsErrorTest, ReturnError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, statfs( _)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));
  int error = StatfsReturnError(FILENAME);
  EXPECT_EQ(GetParam(), error);
}
