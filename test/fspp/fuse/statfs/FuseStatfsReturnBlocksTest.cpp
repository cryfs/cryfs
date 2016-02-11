#include "testutils/FuseStatfsReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseStatfsReturnBlocksTest: public FuseStatfsReturnTest<fsblkcnt_t>, public WithParamInterface<fsblkcnt_t> {
private:
  void set(struct ::statvfs *stat, fsblkcnt_t value) override {
    stat->f_blocks = value;
  }
};
INSTANTIATE_TEST_CASE_P(FuseStatfsReturnBlocksTest, FuseStatfsReturnBlocksTest, Values(
    0,
    10,
    256,
    1024,
    4096
));

TEST_P(FuseStatfsReturnBlocksTest, ReturnedBlocksIsCorrect) {
  struct ::statvfs result = CallStatfsWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.f_blocks);
}
