#include "testutils/FuseTruncateTest.h"

using ::testing::Eq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;

class FuseTruncateSizeTest: public FuseTruncateTest, public WithParamInterface<fspp::num_bytes_t> {
};
INSTANTIATE_TEST_SUITE_P(FuseTruncateSizeTest, FuseTruncateSizeTest, Values(
    fspp::num_bytes_t(0),
    fspp::num_bytes_t(1),
    fspp::num_bytes_t(10),
    fspp::num_bytes_t(1024),
    fspp::num_bytes_t(1024*1024*1024)));


TEST_P(FuseTruncateSizeTest, TruncateFile) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, truncate(Eq(FILENAME), Eq(GetParam())))
    .Times(1).WillOnce(Return());

  TruncateFile(FILENAME, GetParam());
}
