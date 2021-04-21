#include "testutils/FuseAccessTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Eq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::AtLeast;

using namespace fspp::fuse;

class FuseAccessErrorTest: public FuseAccessTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseAccessErrorTest, FuseAccessErrorTest, Values(EACCES, ELOOP, ENAMETOOLONG, ENOENT, ENOTDIR, EROFS, EFAULT, EINVAL, EIO, ENOMEM, ETXTBSY));

TEST_P(FuseAccessErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, access(Eq(FILENAME), testing::_))
    .Times(AtLeast(1)).WillRepeatedly(Throw(FuseErrnoException(GetParam())));

  int error = AccessFileReturnError(FILENAME, 0);
  EXPECT_EQ(GetParam(), error);
}
