#include "testutils/FuseLstatReturnTest.h"

class FuseLstatReturnATimeTest: public FuseLstatReturnTest<time_t> {
public:
  const time_t ATIME1 = 0;
  const time_t ATIME2 = 100;
  const time_t ATIME3 = 1416496809; // current timestamp as of writing the test
  const time_t ATIME4 = 32503680000; // needs a 64bit timestamp
private:
  void set(struct stat *stat, time_t value) override {
    stat->st_atime = value;
  }
};

TEST_F(FuseLstatReturnATimeTest, ReturnedFileAtimeIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(ATIME1);
  EXPECT_EQ(ATIME1, result.st_atime);
}

TEST_F(FuseLstatReturnATimeTest, ReturnedFileAtimeIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(ATIME2);
  EXPECT_EQ(ATIME2, result.st_atime);
}

TEST_F(FuseLstatReturnATimeTest, ReturnedFileAtimeIsCorrect3) {
  struct ::stat result = CallFileLstatWithValue(ATIME3);
  EXPECT_EQ(ATIME3, result.st_atime);
}

TEST_F(FuseLstatReturnATimeTest, ReturnedFileAtimeIsCorrect4) {
  struct ::stat result = CallFileLstatWithValue(ATIME4);
  EXPECT_EQ(ATIME4, result.st_atime);
}
