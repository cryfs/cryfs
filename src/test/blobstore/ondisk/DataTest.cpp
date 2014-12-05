#include "gtest/gtest.h"

#include "blobstore/implementations/ondisk/Data.h"
#include "test/testutils/VirtualTestFile.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using blobstore::ondisk::Data;

class DataTest: public Test {
public:
  bool DataIsZeroes(const Data &data) {
    for (size_t i = 0; i != data.size(); ++ i) {
      if (((char*)data.data())[i] != 0) {
        return false;
      }
    }
    return true;
  }

  void FillData(const VirtualTestFile &fillData, Data *data) {
    ASSERT_EQ(fillData.size(), data->size());
    std::memcpy(data->data(), fillData.data(), fillData.size());
  }

  void CheckData(const VirtualTestFile &expectedData, const Data *data) {
    ASSERT_EQ(expectedData.size(), data->size());
    EXPECT_EQ(0, std::memcmp(expectedData.data(), data->data(), expectedData.size()));
  }
};

class DataTestWithSizeParam: public DataTest, public WithParamInterface<size_t> {
public:
  VirtualTestFile randomData;

  DataTestWithSizeParam(): randomData(GetParam()) {}

  void FillData(Data *data) {
    DataTest::FillData(randomData, data);
  }

  void CheckData(const Data *data) {
    DataTest::CheckData(randomData, data);
  }
};
INSTANTIATE_TEST_CASE_P(DataTestWithSizeParam, DataTestWithSizeParam, Values(0, 1, 1024, 4096, 10*1024*1024));

// Working on a large data area without a crash is a good indicator that we
// are actually working on memory that was validly allocated for us.
TEST_P(DataTestWithSizeParam, WriteAndCheck) {
  Data data(GetParam());

  FillData(&data);
  CheckData(&data);
}

TEST_P(DataTestWithSizeParam, Size) {
  Data data(GetParam());
  EXPECT_EQ(GetParam(), data.size());
}

TEST_F(DataTest, InitializeWithZeroes) {
  Data data(10*1024);
  data.FillWithZeroes();
  EXPECT_TRUE(DataIsZeroes(data));
}

TEST_F(DataTest, FillModifiedDataWithZeroes) {
  Data data(10*1024);
  VirtualTestFile randomData(10*1024);
  FillData(randomData, &data);
  EXPECT_FALSE(DataIsZeroes(data));

  data.FillWithZeroes();
  EXPECT_TRUE(DataIsZeroes(data));
}

//Needs 64bit for representation. This value isn't in the size param list, because the list is also used for read/write checks.
TEST_F(DataTest, LargesizeSize) {
  size_t size = 10L*1024*1024*1024;
  Data data(size);
  EXPECT_EQ(size, data.size());
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

//TODO Test cases for storing/loading
