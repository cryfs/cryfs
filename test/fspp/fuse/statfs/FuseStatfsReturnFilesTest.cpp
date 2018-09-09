#include "testutils/FuseStatfsReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseStatfsReturnFilesTest: public FuseStatfsReturnTest<fsfilcnt64_t>, public WithParamInterface<fsfilcnt64_t> {
private:
  void set(struct ::statvfs *stat, fsfilcnt64_t value) override {
    stat->f_files = value;
  }
};
INSTANTIATE_TEST_CASE_P(FuseStatfsReturnFilesTest, FuseStatfsReturnFilesTest, Values(
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
