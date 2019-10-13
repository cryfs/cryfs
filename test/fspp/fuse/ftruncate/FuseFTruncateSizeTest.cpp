#include "testutils/FuseFTruncateTest.h"

using ::testing::Eq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseFTruncateSizeTest: public FuseFTruncateTest, public WithParamInterface<fspp::num_bytes_t> {
};
INSTANTIATE_TEST_SUITE_P(FuseFTruncateSizeTest, FuseFTruncateSizeTest, Values(
    fspp::num_bytes_t(0),
    fspp::num_bytes_t(1),
    fspp::num_bytes_t(10),
    fspp::num_bytes_t(1024),
    fspp::num_bytes_t(1024*1024*1024)));


TEST_P(FuseFTruncateSizeTest, FTruncateFile) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(*fsimpl, ftruncate(Eq(0), GetParam()))
    .Times(1).WillOnce(Return());
  //Needed to make ::ftruncate system call return successfully
  ReturnIsFileOnFstat(0);

  FTruncateFile(FILENAME, fspp::num_bytes_t(GetParam()));
}
