#include "FuseLstatReturnTest.h"

class FuseLstatReturnPropertyModeTest: public FuseLstatTest {
public:
  const mode_t MODE1 = S_IFREG | S_IRUSR | S_IWGRP | S_IXOTH;
  const mode_t MODE2 = S_IFDIR | S_IWUSR | S_IXGRP | S_IROTH;

  struct stat CallLstatWithValue(mode_t mode) {
    return CallLstatWithImpl([mode] (struct stat *stat) {
      stat->st_mode = mode;
    });
  }
};

TEST_F(FuseLstatReturnPropertyModeTest, ReturnedModeIsCorrect1) {
  struct ::stat result = CallLstatWithValue(MODE1);
  EXPECT_EQ(MODE1, result.st_mode);
}

TEST_F(FuseLstatReturnPropertyModeTest, ReturnedModeIsCorrect2) {
  struct ::stat result = CallLstatWithValue(MODE2);
  EXPECT_EQ(MODE2, result.st_mode);
}
