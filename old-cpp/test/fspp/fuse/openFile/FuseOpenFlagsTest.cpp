#include "testutils/FuseOpenTest.h"

using ::testing::Eq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;

class FuseOpenFlagsTest: public FuseOpenTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseOpenFlagsTest, FuseOpenFlagsTest, Values(O_RDWR, O_RDONLY, O_WRONLY));

TEST_P(FuseOpenFlagsTest, testFlags) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, openFile(Eq(FILENAME), OpenFlagsEq(GetParam())))
    .Times(1).WillOnce(Return(0));

  OpenFile(FILENAME, GetParam());
}
