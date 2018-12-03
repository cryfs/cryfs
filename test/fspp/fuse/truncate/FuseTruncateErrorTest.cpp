#include "testutils/FuseTruncateTest.h"
#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseTruncateErrorTest: public FuseTruncateTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseTruncateErrorTest, FuseTruncateErrorTest, Values(EACCES, EFAULT, EFBIG, EINTR, EINVAL, EIO, EISDIR, ELOOP, ENAMETOOLONG, ENOENT, ENOTDIR, EPERM, EROFS, ETXTBSY));

TEST_P(FuseTruncateErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, truncate(StrEq(FILENAME), _))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = TruncateFileReturnError(FILENAME, fspp::num_bytes_t(0));
  EXPECT_EQ(GetParam(), error);
}
