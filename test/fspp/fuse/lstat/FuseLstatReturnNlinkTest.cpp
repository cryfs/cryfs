#include "testutils/FuseLstatReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnNlinkTest: public FuseLstatReturnTest<nlink_t>, public WithParamInterface<nlink_t> {
private:
  void set(fspp::fuse::STAT *stat, nlink_t value) override {
    stat->st_nlink = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseLstatReturnNlinkTest, FuseLstatReturnNlinkTest, Values(
    1,
    2,
    5,
    100
));

TEST_P(FuseLstatReturnNlinkTest, ReturnedFileNlinkIsCorrect) {
  fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_nlink);
}

TEST_P(FuseLstatReturnNlinkTest, ReturnedDirNlinkIsCorrect) {
  fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_nlink);
}

