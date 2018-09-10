#include "testutils/FuseLstatReturnTest.h"
#include <cpp-utils/system/stat.h>

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnCtimeTest: public FuseLstatReturnTest<time_t>, public WithParamInterface<time_t> {
private:
  void set(struct stat *stat, time_t value) override {
    stat->st_ctim.tv_sec = value;
	stat->st_ctim.tv_nsec = 0;
  }
};
INSTANTIATE_TEST_CASE_P(FuseLstatReturnCtimeTest, FuseLstatReturnCtimeTest, Values(
    0,
    100,
    1416496809, // current timestamp as of writing the test
    32503680000 // needs a 64bit timestamp
));

TEST_P(FuseLstatReturnCtimeTest, ReturnedFileCtimeIsCorrect) {
  struct ::stat result = CallFileLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_ctim.tv_sec);
  EXPECT_EQ(0, result.st_ctim.tv_nsec);
}

TEST_P(FuseLstatReturnCtimeTest, ReturnedDirCtimeIsCorrect) {
  struct ::stat result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_ctim.tv_sec);
  EXPECT_EQ(0, result.st_ctim.tv_nsec);
}
