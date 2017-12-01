#include "testutils/FuseOpenTest.h"

using ::testing::StrEq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;

class FuseOpenFlagsTest: public FuseOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_CASE_P(FuseOpenFlagsTest, FuseOpenFlagsTest, Values(O_RDWR, O_RDONLY, O_WRONLY));

TEST_P(FuseOpenFlagsTest, testFlags) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, openFile(StrEq(FILENAME), OpenFlagsEq(GetParam())))
    .Times(1).WillOnce(Return(0));

  OpenFile(FILENAME, GetParam());
}
