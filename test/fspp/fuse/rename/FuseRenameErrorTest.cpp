#include "testutils/FuseRenameTest.h"
#include "fspp/fuse/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseRenameErrorTest: public FuseRenameTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseRenameErrorTest, FuseRenameErrorTest, Values(EACCES, EBUSY, EDQUOT, EFAULT, EINVAL, EISDIR, ELOOP, EMLINK, ENAMETOOLONG, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENOTEMPTY, EEXIST, EPERM, EROFS, EXDEV, EBADF, ENOTDIR));

TEST_P(FuseRenameErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME1);
  ReturnDoesntExistOnLstat(FILENAME2);
  EXPECT_CALL(fsimpl, rename(StrEq(FILENAME1), StrEq(FILENAME2)))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = RenameReturnError(FILENAME1, FILENAME2);
  EXPECT_EQ(GetParam(), error);
}
