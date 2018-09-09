#include "testutils/FuseMkdirTest.h"

using ::testing::_;
using ::testing::StrEq;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseMkdirModeTest: public FuseMkdirTest, public WithParamInterface<mode_t> {
};
INSTANTIATE_TEST_CASE_P(FuseMkdirModeTest, FuseMkdirModeTest, Values(0, S_IRUSR, S_IRGRP, S_IXOTH, S_IRUSR|S_IRGRP|S_IROTH|S_IRGRP));


TEST_P(FuseMkdirModeTest, Mkdir) {
  ReturnDoesntExistOnLstat(DIRNAME);
  EXPECT_CALL(fsimpl, mkdir(StrEq(DIRNAME), GetParam(), _, _))
    .Times(1).WillOnce(FromNowOnReturnIsDirOnLstat());

  Mkdir(DIRNAME, GetParam());
}
