#include "testutils/FuseUtimensTest.h"
#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::Eq;
using ::testing::Throw;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace fspp::fuse;

class FuseUtimensErrorTest: public FuseUtimensTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseUtimensErrorTest, FuseUtimensErrorTest, Values(EACCES, ENOENT, EPERM, EROFS));

TEST_P(FuseUtimensErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, utimens(Eq(FILENAME), testing::_, testing::_))
    .Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  int error = UtimensReturnError(FILENAME, TIMEVALUE, TIMEVALUE);
  EXPECT_EQ(GetParam(), error);
}
