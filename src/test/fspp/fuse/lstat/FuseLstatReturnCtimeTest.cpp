#include "testutils/FuseLstatReturnTest.h"

class FuseLstatReturnCtimeTest: public FuseLstatReturnTest<time_t> {
public:
  const time_t CTIME1 = 0;
  const time_t CTIME2 = 100;
  const time_t CTIME3 = 1416496809; // current timestamp as of writing the test
  const time_t CTIME4 = 32503680000; // needs a 64bit timestamp
private:
  void set(struct stat *stat, time_t value) override {
    stat->st_ctime = value;
  }
};

TEST_F(FuseLstatReturnCtimeTest, ReturnedFileCtimeIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(CTIME1);
  EXPECT_EQ(CTIME1, result.st_ctime);
}

TEST_F(FuseLstatReturnCtimeTest, ReturnedFileCtimeIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(CTIME2);
  EXPECT_EQ(CTIME2, result.st_ctime);
}

TEST_F(FuseLstatReturnCtimeTest, ReturnedFileCtimeIsCorrect3) {
  struct ::stat result = CallFileLstatWithValue(CTIME3);
  EXPECT_EQ(CTIME3, result.st_ctime);
}

TEST_F(FuseLstatReturnCtimeTest, ReturnedFileCtimeIsCorrect4) {
  struct ::stat result = CallFileLstatWithValue(CTIME4);
  EXPECT_EQ(CTIME4, result.st_ctime);
}
