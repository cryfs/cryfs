#include "testutils/FuseMkdirTest.h"

using ::testing::Eq;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseMkdirModeTest: public FuseMkdirTest, public WithParamInterface<mode_t> {
};
INSTANTIATE_TEST_SUITE_P(FuseMkdirModeTest, FuseMkdirModeTest, Values(0, S_IRUSR, S_IRGRP, S_IXOTH, S_IRUSR|S_IRGRP|S_IROTH|S_IRGRP));


TEST_P(FuseMkdirModeTest, Mkdir) {
  ReturnDoesntExistOnLstat(DIRNAME);
  EXPECT_CALL(*fsimpl, mkdir(Eq(DIRNAME), GetParam(), testing::_, testing::_))
    .Times(1).WillOnce(FromNowOnReturnIsDirOnLstat());

  Mkdir(DIRNAME, GetParam());
}
