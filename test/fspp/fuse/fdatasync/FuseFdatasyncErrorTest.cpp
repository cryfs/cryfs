#include "testutils/FuseFdatasyncTest.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseFdatasyncErrorTest: public FuseFdatasyncTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseFdatasyncErrorTest, FuseFdatasyncErrorTest, Values(EBADF, EIO, EROFS, EINVAL));

TEST_P(FuseFdatasyncErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(fsimpl, fdatasync(0))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = FdatasyncFileReturnError(FILENAME);
  EXPECT_EQ(GetParam(), error);
}
