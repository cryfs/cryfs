#include "testutils/FuseUnlinkTest.h"
#include "fspp/fuse/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseUnlinkErrorTest: public FuseUnlinkTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseUnlinkErrorTest, FuseUnlinkErrorTest, Values(EACCES, EBUSY, EFAULT, EIO, EISDIR, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EPERM, EROFS, EINVAL));

TEST_P(FuseUnlinkErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, unlink(StrEq(FILENAME)))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = UnlinkReturnError(FILENAME);
  EXPECT_EQ(GetParam(), error);
}
