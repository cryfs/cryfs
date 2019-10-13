#include "testutils/FuseUnlinkTest.h"
#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Eq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseUnlinkErrorTest: public FuseUnlinkTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseUnlinkErrorTest, FuseUnlinkErrorTest, Values(EACCES, EBUSY, EFAULT, EIO, EISDIR, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EPERM, EROFS, EINVAL));

TEST_P(FuseUnlinkErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, unlink(Eq(FILENAME)))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = UnlinkReturnError(FILENAME);
  EXPECT_EQ(GetParam(), error);
}
