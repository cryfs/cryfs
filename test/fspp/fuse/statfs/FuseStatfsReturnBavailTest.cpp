#include "testutils/FuseStatfsReturnTest.h"
#include "gtest/gtest.h"
#include <cstdint>
#include <gtest/gtest.h>
#include <sys/statvfs.h>

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseStatfsReturnBavailTest: public FuseStatfsReturnTest<uint64_t>, public WithParamInterface<uint64_t> {
private:
  void set(struct ::statvfs *stat, uint64_t value) override {
    stat->f_bavail = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseStatfsReturnBavailTest, FuseStatfsReturnBavailTest, Values(
    0,
    10,
    256,
    1024,
    4096
));

TEST_P(FuseStatfsReturnBavailTest, ReturnedBavailIsCorrect) {
  const struct ::statvfs result = CallStatfsWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.f_bavail);
}
