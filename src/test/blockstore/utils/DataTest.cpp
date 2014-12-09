#include <blockstore/utils/Data.h>
#include <blockstore/utils/FileDoesntExistException.h>
#include <test/testutils/DataBlockFixture.h>
#include "gtest/gtest.h"

#include "test/testutils/TempFile.h"

#include <fstream>

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::ifstream;
using std::ofstream;

using namespace blockstore;

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

  void FillData(const DataBlockFixture &fillData, Data *data) {
    ASSERT_EQ(fillData.size(), data->size());
    std::memcpy(data->data(), fillData.data(), fillData.size());
  }

  void EXPECT_DATA_CORRECT(const DataBlockFixture &expectedData, const Data &data) {
    ASSERT_EQ(expectedData.size(), data.size());
    EXPECT_EQ(0, std::memcmp(expectedData.data(), data.data(), expectedData.size()));
  }
};

class DataTestWithSizeParam: public DataTest, public WithParamInterface<size_t> {
public:
  DataBlockFixture randomData;

  DataTestWithSizeParam(): randomData(GetParam()) {}

  void FillData(Data *data) {
    DataTest::FillData(randomData, data);
  }

  void StoreData(const bf::path &filepath) {
    ofstream file(filepath.c_str(), std::ios::binary | std::ios::trunc);
    file.write(randomData.data(), randomData.size());
  }

  void EXPECT_STORED_FILE_DATA_CORRECT(const bf::path &filepath) {
    EXPECT_EQ(randomData.size(), bf::file_size(filepath));

    ifstream file(filepath.c_str(), std::ios::binary);
    char *read_data = new char[randomData.size()];
    file.read(read_data, randomData.size());

    EXPECT_EQ(0, std::memcmp(randomData.data(), read_data, randomData.size()));
    delete[] read_data;
  }

  void EXPECT_DATA_CORRECT(const Data &data) {
    DataTest::EXPECT_DATA_CORRECT(randomData, data);
  }
};

INSTANTIATE_TEST_CASE_P(DataTestWithSizeParam, DataTestWithSizeParam, Values(0, 1, 2, 1024, 4096, 10*1024*1024));

// Working on a large data area without a crash is a good indicator that we
// are actually working on memory that was validly allocated for us.
TEST_P(DataTestWithSizeParam, WriteAndCheck) {
  Data data(GetParam());

  FillData(&data);
  EXPECT_DATA_CORRECT(data);
}

TEST_P(DataTestWithSizeParam, Size) {
  Data data(GetParam());
  EXPECT_EQ(GetParam(), data.size());
}

TEST_P(DataTestWithSizeParam, CheckStoredFile) {
  Data data(GetParam());
  FillData(&data);

  TempFile file;
  data.StoreToFile(file.path());

  EXPECT_STORED_FILE_DATA_CORRECT(file.path());
}

TEST_P(DataTestWithSizeParam, CheckLoadedData) {
  TempFile file;
  StoreData(file.path());

  Data data = Data::LoadFromFile(file.path());

  EXPECT_DATA_CORRECT(data);
}

TEST_P(DataTestWithSizeParam, StoreDoesntChangeData) {
  Data data(GetParam());
  FillData(&data);

  TempFile file;
  data.StoreToFile(file.path());

  EXPECT_DATA_CORRECT(data);
}

TEST_P(DataTestWithSizeParam, StoreAndLoad) {
  Data data(GetParam());
  FillData(&data);

  TempFile file;
  data.StoreToFile(file.path());
  Data loaded_data = Data::LoadFromFile(file.path());

  EXPECT_DATA_CORRECT(loaded_data);
}

TEST_F(DataTest, InitializeWithZeroes) {
  Data data(10*1024);
  data.FillWithZeroes();
  EXPECT_TRUE(DataIsZeroes(data));
}

TEST_F(DataTest, FillModifiedDataWithZeroes) {
  Data data(10*1024);
  DataBlockFixture randomData(10*1024);
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

TEST_F(DataTest, LoadingNonexistingFile) {
  TempFile file(false); // Pass false to constructor, so the tempfile is not created
  EXPECT_THROW(
    Data::LoadFromFile(file.path()),
    FileDoesntExistException
  );
}
