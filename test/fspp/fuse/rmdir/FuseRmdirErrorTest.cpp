#include "testutils/FuseRmdirTest.h"
#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Eq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseRmdirErrorTest: public FuseRmdirTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseRmdirErrorTest, FuseRmdirErrorTest, Values(EACCES, EBUSY, EFAULT, EINVAL, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, ENOTEMPTY, EPERM, EROFS));

TEST_P(FuseRmdirErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsDirOnLstat(DIRNAME);
  EXPECT_CALL(*fsimpl, rmdir(Eq(DIRNAME)))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = RmdirReturnError(DIRNAME);
  EXPECT_EQ(GetParam(), error);
}
