#include "testutils/FuseMkdirTest.h"
#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseMkdirErrorTest: public FuseMkdirTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseMkdirErrorTest, FuseMkdirErrorTest, Values(EACCES, EDQUOT, EEXIST, EFAULT, ELOOP, EMLINK, ENAMETOOLONG, ENOENT, ENOMEM, ENOSPC, ENOTDIR, EPERM, EROFS, EBADF));

TEST_P(FuseMkdirErrorTest, ReturnedErrorIsCorrect) {
  ReturnDoesntExistOnLstat(DIRNAME);
  EXPECT_CALL(fsimpl, mkdir(StrEq(DIRNAME), _))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int retval = MkdirAllowError(DIRNAME, 0);
  EXPECT_EQ(GetParam(), errno);
  EXPECT_EQ(-1, retval);
}
