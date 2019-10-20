#include "testutils/FuseStatfsReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseStatfsReturnFilesTest: public FuseStatfsReturnTest<uint64_t>, public WithParamInterface<uint64_t> {
private:
  void set(struct ::statvfs *stat, uint64_t value) override {
    stat->f_files = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseStatfsReturnFilesTest, FuseStatfsReturnFilesTest, Values(
    0,
    10,
    256,
    1024,
    4096
));

TEST_P(FuseStatfsReturnFilesTest, ReturnedFilesIsCorrect) {
  struct ::statvfs result = CallStatfsWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.f_files);
}
