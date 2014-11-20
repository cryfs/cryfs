#include "testutils/FuseLstatReturnTest.h"

class FuseLstatReturnMtimeTest: public FuseLstatReturnTest<time_t> {
public:
  const time_t MTIME1 = 0;
  const time_t MTIME2 = 100;
  const time_t MTIME3 = 1416496809; // current timestamp as of writing the test
  const time_t MTIME4 = 32503680000; // needs a 64bit timestamp
private:
  void set(struct stat *stat, time_t value) override {
    stat->st_mtime = value;
  }
};

TEST_F(FuseLstatReturnMtimeTest, ReturnedFileMtimeIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(MTIME1);
  EXPECT_EQ(MTIME1, result.st_mtime);
}

TEST_F(FuseLstatReturnMtimeTest, ReturnedFileMtimeIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(MTIME2);
  EXPECT_EQ(MTIME2, result.st_mtime);
}

TEST_F(FuseLstatReturnMtimeTest, ReturnedFileMtimeIsCorrect3) {
  struct ::stat result = CallFileLstatWithValue(MTIME3);
  EXPECT_EQ(MTIME3, result.st_mtime);
}

TEST_F(FuseLstatReturnMtimeTest, ReturnedFileMtimeIsCorrect4) {
  struct ::stat result = CallFileLstatWithValue(MTIME4);
  EXPECT_EQ(MTIME4, result.st_mtime);
}
