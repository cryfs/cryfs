#include "testutils/FuseLstatReturnTest.h"

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseLstatReturnModeTest: public FuseLstatTest, public WithParamInterface<mode_t> {
public:
  fspp::fuse::STAT CallLstatWithValue(mode_t mode) {
    return CallLstatWithImpl([mode] (fspp::fuse::STAT *stat) {
      stat->st_mode = mode;
    });
  }
};
INSTANTIATE_TEST_SUITE_P(FuseLstatReturnModeTest, FuseLstatReturnModeTest, Values(
    S_IFREG,
    S_IFDIR,
    S_IFREG | S_IRUSR | S_IWGRP | S_IXOTH, // a file with some access bits set
    S_IFDIR | S_IWUSR | S_IXGRP | S_IROTH  // a dir with some access bits set
));

TEST_P(FuseLstatReturnModeTest, ReturnedModeIsCorrect) {
  fspp::fuse::STAT result = CallLstatWithValue(GetParam());
  EXPECT_EQ(GetParam(), result.st_mode);
}
