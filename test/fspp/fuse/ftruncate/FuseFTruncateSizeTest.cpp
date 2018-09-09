#include "testutils/FuseFTruncateTest.h"

using ::testing::Eq;
using ::testing::Return;
using ::testing::WithParamInterface;
using ::testing::Values;

class FuseFTruncateSizeTest: public FuseFTruncateTest, public WithParamInterface<off_t> {
};
INSTANTIATE_TEST_CASE_P(FuseFTruncateSizeTest, FuseFTruncateSizeTest, Values(0, 1, 10, 1024, 1024*1024*1024));


TEST_P(FuseFTruncateSizeTest, FTruncateFile) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(fsimpl, ftruncate(Eq(0), GetParam()))
    .Times(1).WillOnce(Return());
  //Needed to make ::ftruncate system call return successfully
  ReturnIsFileOnFstat(0);

  FTruncateFile(FILENAME, GetParam());
}
