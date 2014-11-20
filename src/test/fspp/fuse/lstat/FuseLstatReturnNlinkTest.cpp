#include "FuseLstatReturnTest.h"

class FuseLstatReturnPropertyNlinkTest: public FuseLstatReturnPropertyTest<nlink_t> {
public:
  const nlink_t NLINK1 = 1;
  const nlink_t NLINK2 = 5;
private:
  void set(struct stat *stat, nlink_t value) override {
    stat->st_nlink = value;
  }
};

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedFileNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedFileNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedDirNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedDirNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}
