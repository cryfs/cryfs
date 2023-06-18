#include "testutils/FuseStatfsReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseStatfsReturnNamemaxTest: public FuseStatfsReturnTest<unsigned long>, public WithParamInterface<unsigned long> {
private:
  void set(struct ::statvfs *stat, unsigned long value) override {
    stat->f_namemax = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseStatfsReturnNamemaxTest, FuseStatfsReturnNamemaxTest, Values(
    0,
    10,
    256,
    1024,
    4096
));

TEST_P(FuseStatfsReturnNamemaxTest, ReturnedNamemaxIsCorrect) {
  struct ::statvfs result = CallStatfsWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.f_namemax);
}
