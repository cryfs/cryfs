#include "testutils/FuseLstatTest.h"

#include "fspp/impl/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using fspp::FuseErrnoException;

class FuseLstatErrorTest: public FuseLstatTest, public WithParamInterface<int> {
public:
};
INSTANTIATE_TEST_CASE_P(LstatErrorCodes, FuseLstatErrorTest, Values(EACCES, EBADF, EFAULT, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EOVERFLOW, EINVAL, ENOTDIR));

TEST_F(FuseLstatErrorTest, ReturnNoError) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).Times(1).WillOnce(ReturnIsFile);
  int retval = LstatPathAllowErrors(FILENAME);
  EXPECT_EQ(retval, 0);
}

TEST_P(FuseLstatErrorTest, ReturnError) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));
  int retval = LstatPathAllowErrors(FILENAME);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(GetParam(), errno);
}
