#include "testutils/FuseLstatReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnNlinkTest: public FuseLstatReturnTest<nlink_t>, public WithParamInterface<nlink_t> {
private:
  void set(struct stat *stat, nlink_t value) override {
    stat->st_nlink = value;
  }
};
INSTANTIATE_TEST_CASE_P(FuseLstatReturnNlinkTest, FuseLstatReturnNlinkTest, Values(
    1,
    2,
    5,
    100
));

TEST_P(FuseLstatReturnNlinkTest, ReturnedFileNlinkIsCorrect) {
  struct ::stat result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_nlink);
}

TEST_P(FuseLstatReturnNlinkTest, ReturnedDirNlinkIsCorrect) {
  struct ::stat result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_nlink);
}

