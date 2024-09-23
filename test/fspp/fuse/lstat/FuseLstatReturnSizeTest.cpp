#include "fspp/fs_interface/Types.h"
#include "fspp/fuse/stat_compatibility.h"
#include "testutils/FuseLstatReturnTest.h"
#include "gtest/gtest.h"
#include <gtest/gtest.h>

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnSizeTest: public FuseLstatReturnTest<fspp::num_bytes_t>, public WithParamInterface<fspp::num_bytes_t> {
private:
  void set(fspp::fuse::STAT *stat, fspp::num_bytes_t value) override {
    stat->st_size = value.value();
  }
};
INSTANTIATE_TEST_SUITE_P(FuseLstatReturnSizeTest, FuseLstatReturnSizeTest, Values(
  fspp::num_bytes_t(0),
  fspp::num_bytes_t(1),
  fspp::num_bytes_t(4096),
  fspp::num_bytes_t(1024*1024*1024)
));

TEST_P(FuseLstatReturnSizeTest, ReturnedFileSizeIsCorrect) {
  const fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), fspp::num_bytes_t(result.st_size));
}

TEST_P(FuseLstatReturnSizeTest, ReturnedDirSizeIsCorrect) {
  const fspp::fuse::STAT result = CallDirLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), fspp::num_bytes_t(result.st_size));
}
