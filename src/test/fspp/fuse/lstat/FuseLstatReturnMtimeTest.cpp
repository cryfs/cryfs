#include "testutils/FuseLstatReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnMtimeTest: public FuseLstatReturnTest<time_t>, public WithParamInterface<time_t> {
private:
  void set(struct stat *stat, time_t value) override {
    stat->st_mtime = value;
  }
};
INSTANTIATE_TEST_CASE_P(FuseLstatReturnMtimeTest, FuseLstatReturnMtimeTest, Values(
    0,
    100,
    1416496809, // current timestamp as of writing the test
    32503680000 // needs a 64bit timestamp
));

TEST_P(FuseLstatReturnMtimeTest, ReturnedFileMtimeIsCorrect) {
  struct ::stat result = CallFileLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_mtime);
}

TEST_P(FuseLstatReturnMtimeTest, ReturnedDirMtimeIsCorrect) {
  struct ::stat result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_mtime);
}
