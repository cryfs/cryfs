#include "testutils/FuseMkdirTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Eq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseMkdirErrorTest: public FuseMkdirTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseMkdirErrorTest, FuseMkdirErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, ELOOP, EMLINK, ENAMETOOLONG, ENOENT, ENOMEM, ENOSPC, ENOTDIR, EPERM, EROFS, EBADF));

TEST_F(FuseMkdirErrorTest, NoError) {
  ReturnDoesntExistOnLstat(DIRNAME);
  EXPECT_CALL(*fsimpl, mkdir(Eq(DIRNAME), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(FromNowOnReturnIsDirOnLstat());

  int error = MkdirReturnError(DIRNAME, 0);
  EXPECT_EQ(0, error);
}

TEST_P(FuseMkdirErrorTest, ReturnedErrorIsCorrect) {
  ReturnDoesntExistOnLstat(DIRNAME);
  EXPECT_CALL(*fsimpl, mkdir(Eq(DIRNAME), testing::_, testing::_, testing::_))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = MkdirReturnError(DIRNAME, 0);
  EXPECT_EQ(GetParam(), error);
}
