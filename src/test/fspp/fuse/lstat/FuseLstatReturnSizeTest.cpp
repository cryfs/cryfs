#include "FuseLstatReturnTest.h"

class FuseLstatReturnPropertySizeTest: public FuseLstatReturnPropertyTest<off_t> {
public:
  const off_t SIZE1 = 0;
  const off_t SIZE2 = 4096;
  const off_t SIZE3 = 1024*1024*1024;
private:
  void set(struct stat *stat, off_t value) override {
    stat->st_size = value;
  }
};

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedFileSizeIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(SIZE1);
  EXPECT_EQ(SIZE1, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedFileSizeIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(SIZE2);
  EXPECT_EQ(SIZE2, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedFileSizeIsCorrect3) {
  struct ::stat result = CallDirLstatWithValue(SIZE3);
  EXPECT_EQ(SIZE3, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedDirSizeIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(SIZE1);
  EXPECT_EQ(SIZE1, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedDirSizeIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(SIZE2);
  EXPECT_EQ(SIZE2, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedDirSizeIsCorrect3) {
  struct ::stat result = CallDirLstatWithValue(SIZE3);
  EXPECT_EQ(SIZE3, result.st_size);
}
