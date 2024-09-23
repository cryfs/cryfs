#include "fspp/fuse/stat_compatibility.h"
#include "testutils/FuseLstatReturnTest.h"
#include "gtest/gtest.h"
#include <gtest/gtest.h>
#include <sys/types.h>

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnUidTest: public FuseLstatReturnTest<uid_t>, public WithParamInterface<uid_t> {
private:
  void set(fspp::fuse::STAT *stat, uid_t value) override {
    stat->st_uid = value;
  }
};
INSTANTIATE_TEST_SUITE_P(FuseLstatReturnUidTest, FuseLstatReturnUidTest, Values(
    0,
    10
));

TEST_P(FuseLstatReturnUidTest, ReturnedFileUidIsCorrect) {
  const fspp::fuse::STAT result = CallFileLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_uid);
}

TEST_P(FuseLstatReturnUidTest, ReturnedDirUidIsCorrect) {
  const fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_uid);
}
