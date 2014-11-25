#include "testutils/FuseCreateAndOpenTest.h"

#include "fspp/impl/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using ::testing::Throw;
using ::testing::StrEq;
using ::testing::_;

using namespace fspp;

class FuseCreateAndOpenErrorTest: public FuseCreateAndOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseCreateAndOpenErrorTest, FuseCreateAndOpenErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, EFBIG, EINTR, EOVERFLOW, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOPNOTSUPP, EPERM, EROFS, ETXTBSY, EWOULDBLOCK, EBADF, ENOTDIR));

TEST_F(FuseCreateAndOpenErrorTest, ReturnNoError) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, createAndOpenFile(StrEq(FILENAME), _)).Times(1).WillOnce(Return(1));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(1);

  errno = 0;
  int retval = CreateAndOpenFileAllowError(FILENAME, O_RDONLY);
  EXPECT_EQ(errno, 0);
  EXPECT_GE(retval, 0);
}

TEST_P(FuseCreateAndOpenErrorTest, ReturnError) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, createAndOpenFile(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int retval = CreateAndOpenFileAllowError(FILENAME, O_RDONLY);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(GetParam(), errno);
}
