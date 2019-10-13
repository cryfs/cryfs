#include "testutils/FuseLstatReturnTest.h"
#include <cpp-utils/system/stat.h>

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnMtimeTest: public FuseLstatReturnTest<time_t>, public WithParamInterface<time_t> {
private:
  void set(fspp::fuse::STAT *stat, time_t value) override {
    stat->st_mtim.tv_sec = value;
	stat->st_mtim.tv_nsec = 0;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseLstatReturnMtimeTest, FuseLstatReturnMtimeTest, Values(
    0,
    100,
    1416496809, // current timestamp as of writing the test
    32503680000 // needs a 64bit timestamp
));

TEST_P(FuseLstatReturnMtimeTest, ReturnedFileMtimeIsCorrect) {
  fspp::fuse::STAT result = CallFileLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_mtim.tv_sec);
  EXPECT_EQ(0, result.st_mtim.tv_nsec);
}

TEST_P(FuseLstatReturnMtimeTest, ReturnedDirMtimeIsCorrect) {
  fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_mtim.tv_sec);
  EXPECT_EQ(0, result.st_mtim.tv_nsec);
}
