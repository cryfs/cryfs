#include "testutils/FuseLstatReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnSizeTest: public FuseLstatReturnTest<off_t>, public WithParamInterface<off_t> {
private:
  void set(struct stat *stat, off_t value) override {
    stat->st_size = value;
  }
};
INSTANTIATE_TEST_CASE_P(FuseLstatReturnSizeTest, FuseLstatReturnSizeTest, Values(
    0,
    1,
    4096,
    1024*1024*1024
));

TEST_P(FuseLstatReturnSizeTest, ReturnedFileSizeIsCorrect) {
  struct ::stat result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_size);
}

TEST_P(FuseLstatReturnSizeTest, ReturnedDirSizeIsCorrect) {
  struct ::stat result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_size);
}
