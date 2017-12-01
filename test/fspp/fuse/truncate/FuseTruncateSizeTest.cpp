#include "testutils/FuseTruncateTest.h"

using ::testing::StrEq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseTruncateSizeTest: public FuseTruncateTest, public WithParamInterface<off_t> {
};
INSTANTIATE_TEST_CASE_P(FuseTruncateSizeTest, FuseTruncateSizeTest, Values(0, 1, 10, 1024, 1024*1024*1024));


TEST_P(FuseTruncateSizeTest, TruncateFile) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, truncate(StrEq(FILENAME), GetParam()))
    .Times(1).WillOnce(Return());

  TruncateFile(FILENAME, GetParam());
}
