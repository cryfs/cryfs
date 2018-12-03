#include "testutils/FuseLstatTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::AtLeast;

using fspp::fuse::FuseErrnoException;

class FuseLstatErrorTest: public FuseLstatTest, public WithParamInterface<int> {
public:
};
INSTANTIATE_TEST_CASE_P(LstatErrorCodes, FuseLstatErrorTest, Values(EACCES, EBADF, EFAULT, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EOVERFLOW, EINVAL, ENOTDIR));

TEST_F(FuseLstatErrorTest, ReturnNoError) {
  EXPECT_CALL(*fsimpl, lstat(StrEq(FILENAME), _)).Times(AtLeast(1)).WillRepeatedly(ReturnIsFile);
  errno = 0;
  int error = LstatPathReturnError(FILENAME);
  EXPECT_EQ(0, error);
}

TEST_P(FuseLstatErrorTest, ReturnError) {
  EXPECT_CALL(*fsimpl, lstat(StrEq(FILENAME), _)).Times(AtLeast(1)).WillRepeatedly(Throw(FuseErrnoException(GetParam())));
  int error = LstatPathReturnError(FILENAME);
  EXPECT_EQ(GetParam(), error);
}
