#include "testutils/FuseFTruncateTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseFTruncateErrorTest: public FuseFTruncateTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFTruncateErrorTest, FuseFTruncateErrorTest, Values(EACCES, EFAULT, EFBIG, EINTR, EINVAL, EIO, EISDIR, ELOOP, ENAMETOOLONG, ENOENT, ENOTDIR, EPERM, EROFS, ETXTBSY, EBADF));

TEST_P(FuseFTruncateErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(*fsimpl, ftruncate(0, testing::_))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));
  //Needed to make ::ftruncate system call return successfully
  ReturnIsFileOnFstat(0);

  int error = FTruncateFileReturnError(FILENAME, fspp::num_bytes_t(0));
  EXPECT_EQ(GetParam(), error);
}
