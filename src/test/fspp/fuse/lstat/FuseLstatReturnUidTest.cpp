#include "FuseLstatReturnTest.h"

class FuseLstatReturnPropertyUidTest: public FuseLstatReturnPropertyTest<uid_t> {
public:
  const uid_t UID1 = 0;
  const uid_t UID2 = 10;
private:
  void set(struct stat *stat, uid_t value) override {
    stat->st_uid = value;
  }
};

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedFileUidIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedFileUidIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedDirUidIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedDirUidIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}
