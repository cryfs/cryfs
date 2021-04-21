#include "testutils/FuseCreateAndOpenTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using ::testing::Throw;
using ::testing::Eq;

using namespace fspp::fuse;

class FuseCreateAndOpenErrorTest: public FuseCreateAndOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseCreateAndOpenErrorTest, FuseCreateAndOpenErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, EFBIG, EINTR, EOVERFLOW, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOPNOTSUPP, EPERM, EROFS, ETXTBSY, EWOULDBLOCK, EBADF, ENOTDIR));

TEST_F(FuseCreateAndOpenErrorTest, ReturnNoError) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(FILENAME), testing::_, testing::_, testing::_)).Times(1).WillOnce(Return(1));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(1);

  int error = CreateAndOpenFileReturnError(FILENAME, O_RDONLY);
  EXPECT_EQ(0, error);
}

TEST_P(FuseCreateAndOpenErrorTest, ReturnError) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(FILENAME), testing::_, testing::_, testing::_)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = CreateAndOpenFileReturnError(FILENAME, O_RDONLY);
  EXPECT_EQ(GetParam(), error);
}
