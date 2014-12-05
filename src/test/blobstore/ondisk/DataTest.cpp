#include "gtest/gtest.h"

#include "blobstore/implementations/ondisk/Data.h"

using ::testing::Test;

using blobstore::ondisk::Data;

class DataTest: public Test {
public:
  void FillData(char *data, size_t size) {
    for (size_t i = 0; i < size; ++i) {
      data[i] = i;
    }
  }

  void CheckData(char *data, size_t size) {
    for (size_t i = 0; i < size; ++i) {
      EXPECT_EQ((char)i, data[i]);
    }
  }
};

TEST_F(DataTest, EmptyData) {
  Data data(0);
}

TEST_F(DataTest, OneByteData) {
  Data data(1);
  ((char*)data.data())[0] = 0x3E;

  EXPECT_EQ(0x3E, ((char*)data.data())[0]);
}

TEST_F(DataTest, MidsizeData) {
  Data data(4096);

  FillData((char*)data.data(), 4096);
  CheckData((char*)data.data(), 4096);
}

// Working on a large data area without a crash is a good indicator that we
// are actually working on memory that was validly allocated for us.
TEST_F(DataTest, LargeData) {
  Data data(10 * 1024 * 1024);

  FillData((char*)data.data(), 10 * 1024 * 1024);
  CheckData((char*)data.data(), 10 * 1024 * 1024);
}

// This test doesn't ensure that the Data class gives the memory region free,
// but it is a good indicator.
TEST_F(DataTest, InaccessibleAfterDeletion) {
  Data *data = new Data(1);
  ((char*)data->data())[0] = 0x3E; // Access data byte 0

  delete data;

  EXPECT_DEATH(
      ((char*)data->data())[0] = 0x3E,
      ""
  );
}
