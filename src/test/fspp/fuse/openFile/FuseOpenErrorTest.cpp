#include "testutils/FuseOpenTest.h"

#include "fspp/impl/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using ::testing::Throw;
using ::testing::StrEq;
using ::testing::_;

using namespace fspp;

class FuseOpenErrorTest: public FuseOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(OpenErrorCodes, FuseOpenErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, EFBIG, EINTR, EOVERFLOW, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOPNOTSUPP, EPERM, EROFS, ETXTBSY, EWOULDBLOCK, EBADF, ENOTDIR));

TEST_F(FuseOpenErrorTest, ReturnNoError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, openFile(StrEq(FILENAME), _)).Times(1).WillOnce(Return(1));
  errno = 0;
  int retval = OpenFileAllowError(FILENAME, O_RDONLY);
  EXPECT_EQ(errno, 0);
  EXPECT_GE(retval, 0);
}

TEST_P(FuseOpenErrorTest, ReturnError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, openFile(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));
  int retval = OpenFileAllowError(FILENAME, O_RDONLY);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(GetParam(), errno);
}
