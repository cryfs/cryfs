#include "testutils/FuseRmdirTest.h"
#include "fspp/fuse/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseRmdirErrorTest: public FuseRmdirTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseRmdirErrorTest, FuseRmdirErrorTest, Values(EACCES, EBUSY, EFAULT, EINVAL, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, ENOTEMPTY, EPERM, EROFS));

TEST_P(FuseRmdirErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsDirOnLstat(DIRNAME);
  EXPECT_CALL(fsimpl, rmdir(StrEq(DIRNAME)))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = RmdirReturnError(DIRNAME);
  EXPECT_EQ(GetParam(), error);
}
