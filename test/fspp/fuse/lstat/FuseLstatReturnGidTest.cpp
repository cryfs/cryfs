#include "testutils/FuseLstatReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnGidTest: public FuseLstatReturnTest<gid_t>, public WithParamInterface<gid_t> {
private:
  void set(fspp::fuse::STAT *stat, gid_t value) override {
    stat->st_gid = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseLstatReturnGidTest, FuseLstatReturnGidTest, Values(
    0,
    10
));

TEST_P(FuseLstatReturnGidTest, ReturnedFileGidIsCorrect) {
  fspp::fuse::STAT result = CallFileLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_gid);
}

TEST_P(FuseLstatReturnGidTest, ReturnedDirGidIsCorrect) {
  fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_gid);
}
