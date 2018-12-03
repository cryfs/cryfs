#include "testutils/FuseUtimensTest.h"
#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseUtimensErrorTest: public FuseUtimensTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseUtimensErrorTest, FuseUtimensErrorTest, Values(EACCES, ENOENT, EPERM, EROFS));

TEST_P(FuseUtimensErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, utimens(StrEq(FILENAME), _, _))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = UtimensReturnError(FILENAME, TIMEVALUE, TIMEVALUE);
  EXPECT_EQ(GetParam(), error);
}
