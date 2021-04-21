#include "testutils/FuseOpenTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using ::testing::Throw;
using ::testing::Eq;

using namespace fspp::fuse;

class FuseOpenErrorTest: public FuseOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseOpenErrorTest, FuseOpenErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, EFBIG, EINTR, EOVERFLOW, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOPNOTSUPP, EPERM, EROFS, ETXTBSY, EWOULDBLOCK, EBADF, ENOTDIR));

TEST_F(FuseOpenErrorTest, ReturnNoError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, openFile(Eq(FILENAME), testing::_)).Times(1).WillOnce(Return(1));
  errno = 0;
  int error = OpenFileReturnError(FILENAME, O_RDONLY);
  EXPECT_EQ(0, error);
}

TEST_P(FuseOpenErrorTest, ReturnError) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, openFile(Eq(FILENAME), testing::_)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));
  int error = OpenFileReturnError(FILENAME, O_RDONLY);
  EXPECT_EQ(GetParam(), error);
}
