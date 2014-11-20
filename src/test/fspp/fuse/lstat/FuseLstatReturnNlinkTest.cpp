#include "testutils/FuseLstatReturnTest.h"

class FuseLstatReturnNlinkTest: public FuseLstatReturnTest<nlink_t> {
public:
  const nlink_t NLINK1 = 1;
  const nlink_t NLINK2 = 5;
private:
  void set(struct stat *stat, nlink_t value) override {
    stat->st_nlink = value;
  }
};

TEST_F(FuseLstatReturnNlinkTest, ReturnedFileNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatReturnNlinkTest, ReturnedFileNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}

TEST_F(FuseLstatReturnNlinkTest, ReturnedDirNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatReturnNlinkTest, ReturnedDirNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}
