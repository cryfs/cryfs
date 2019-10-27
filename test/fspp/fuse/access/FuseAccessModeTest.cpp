#include "testutils/FuseAccessTest.h"

using ::testing::Eq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseAccessModeTest: public FuseAccessTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseAccessModeTest, FuseAccessModeTest, Values(0, F_OK, R_OK, W_OK, X_OK, R_OK|W_OK, W_OK|X_OK, R_OK|X_OK, R_OK|W_OK|X_OK));


TEST_P(FuseAccessModeTest, AccessFile) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, access(Eq(FILENAME), GetParam()))
    .Times(1).WillOnce(Return());

  AccessFile(FILENAME, GetParam());
}
