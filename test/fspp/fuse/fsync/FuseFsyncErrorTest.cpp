#include "testutils/FuseFsyncTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseFsyncErrorTest: public FuseFsyncTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFsyncErrorTest, FuseFsyncErrorTest, Values(EBADF, EIO, EROFS, EINVAL));

TEST_P(FuseFsyncErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(*fsimpl, fsync(0))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = FsyncFileReturnError(FILENAME);
  EXPECT_EQ(GetParam(), error);
}
