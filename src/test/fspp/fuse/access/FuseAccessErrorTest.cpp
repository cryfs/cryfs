#include "testutils/FuseAccessTest.h"
#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "fspp/fuse/FuseErrnoException.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseAccessErrorTest: public FuseAccessTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseAccessErrorTest, FuseAccessErrorTest, Values(EACCES, ELOOP, ENAMETOOLONG, ENOENT, ENOTDIR, EROFS, EFAULT, EINVAL, EIO, ENOMEM, ETXTBSY));

TEST_P(FuseAccessErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, access(StrEq(FILENAME), _))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int retval = AccessFileAllowError(FILENAME, 0);
  EXPECT_EQ(GetParam(), errno);
  EXPECT_EQ(-1, retval);
}
