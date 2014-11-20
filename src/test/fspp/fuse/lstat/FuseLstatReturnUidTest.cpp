#include "testutils/FuseLstatReturnTest.h"

class FuseLstatReturnUidTest: public FuseLstatReturnTest<uid_t> {
public:
  const uid_t UID1 = 0;
  const uid_t UID2 = 10;
private:
  void set(struct stat *stat, uid_t value) override {
    stat->st_uid = value;
  }
};

TEST_F(FuseLstatReturnUidTest, ReturnedFileUidIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatReturnUidTest, ReturnedFileUidIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}

TEST_F(FuseLstatReturnUidTest, ReturnedDirUidIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatReturnUidTest, ReturnedDirUidIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}
