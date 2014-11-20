#include "FuseLstatReturnTest.h"

class FuseLstatReturnPropertyGidTest: public FuseLstatReturnPropertyTest<gid_t> {
public:
  const gid_t GID1 = 0;
  const gid_t GID2 = 10;
private:
  void set(struct stat *stat, gid_t value) override {
    stat->st_gid = value;
  }
};

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedFileGidIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(GID1);
  EXPECT_EQ(GID1, result.st_gid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedFileGidIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(GID2);
  EXPECT_EQ(GID2, result.st_gid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedDirGidIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(GID1);
  EXPECT_EQ(GID1, result.st_gid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedDirGidIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(GID2);
  EXPECT_EQ(GID2, result.st_gid);
}
