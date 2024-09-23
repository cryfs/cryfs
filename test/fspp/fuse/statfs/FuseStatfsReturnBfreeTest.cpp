#include "testutils/FuseStatfsReturnTest.h"
#include "gtest/gtest.h"
#include <cstdint>
#include <gtest/gtest.h>
#include <sys/statvfs.h>

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseStatfsReturnBfreeTest: public FuseStatfsReturnTest<uint64_t>, public WithParamInterface<uint64_t> {
private:
  void set(struct ::statvfs *stat, uint64_t value) override {
    stat->f_bfree = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseStatfsReturnBfreeTest, FuseStatfsReturnBfreeTest, Values(
    0,
    10,
    256,
    1024,
    4096
));

TEST_P(FuseStatfsReturnBfreeTest, ReturnedBfreeIsCorrect) {
  const struct ::statvfs result = CallStatfsWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.f_bfree);
}
