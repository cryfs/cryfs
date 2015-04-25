#include "../../data/DataFixture.h"
#include "../../data/Data.h"
#include "google/gtest/gtest.h"

#include "../../tempfile/TempFile.h"

#include <fstream>

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using cpputils::TempFile;

using std::ifstream;
using std::ofstream;

namespace bf = boost::filesystem;

using namespace cpputils;

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
};

class DataTestWithSizeParam: public DataTest, public WithParamInterface<size_t> {
public:
  Data randomData;

  DataTestWithSizeParam(): randomData(DataFixture::generate(GetParam())) {}

  static void StoreData(const Data &data, const bf::path &filepath) {
    ofstream file(filepath.c_str(), std::ios::binary | std::ios::trunc);
    file.write((char*)data.data(), data.size());
  }

  static void EXPECT_STORED_FILE_DATA_CORRECT(const Data &data, const bf::path &filepath) {
    EXPECT_EQ(data.size(), bf::file_size(filepath));

    ifstream file(filepath.c_str(), std::ios::binary);
    char *read_data = new char[data.size()];
    file.read(read_data, data.size());

    EXPECT_EQ(0, std::memcmp(data.data(), read_data, data.size()));
    delete[] read_data;
  }
};

INSTANTIATE_TEST_CASE_P(DataTestWithSizeParam, DataTestWithSizeParam, Values(0, 1, 2, 1024, 4096, 10*1024*1024));

TEST_P(DataTestWithSizeParam, ZeroInitializedDataIsDifferentToRandomData) {
  if (GetParam() != 0) {
    Data data(GetParam());
    data.FillWithZeroes();
    EXPECT_NE(randomData, data);
  }
}

// Working on a large data area without a crash is a good indicator that we
// are actually working on memory that was validly allocated for us.
TEST_P(DataTestWithSizeParam, WriteAndCheck) {
  Data data = randomData.copy();
  EXPECT_EQ(randomData, data);
}

TEST_P(DataTestWithSizeParam, Size) {
  Data data(GetParam());
  EXPECT_EQ(GetParam(), data.size());
}

TEST_P(DataTestWithSizeParam, CheckStoredFile) {
  TempFile file;
  randomData.StoreToFile(file.path());

  EXPECT_STORED_FILE_DATA_CORRECT(randomData, file.path());
}

TEST_P(DataTestWithSizeParam, CheckLoadedData) {
  TempFile file;
  StoreData(randomData, file.path());

  Data data = Data::LoadFromFile(file.path()).value();

  EXPECT_EQ(randomData, data);
}

TEST_P(DataTestWithSizeParam, StoreDoesntChangeData) {
  Data data = randomData.copy();

  TempFile file;
  data.StoreToFile(file.path());

  EXPECT_EQ(randomData, data);
}

TEST_P(DataTestWithSizeParam, StoreAndLoad) {
  TempFile file;
  randomData.StoreToFile(file.path());
  Data loaded_data = Data::LoadFromFile(file.path()).value();

  EXPECT_EQ(randomData, loaded_data);
}

TEST_P(DataTestWithSizeParam, Copy) {
  Data copy = randomData.copy();
  EXPECT_EQ(randomData, copy);
}

TEST_F(DataTest, InitializeWithZeroes) {
  Data data(10*1024);
  data.FillWithZeroes();
  EXPECT_TRUE(DataIsZeroes(data));
}

TEST_F(DataTest, FillModifiedDataWithZeroes) {
  Data data = DataFixture::generate(10*1024);
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

TEST_F(DataTest, LoadingNonexistingFile) {
  TempFile file(false); // Pass false to constructor, so the tempfile is not created
  EXPECT_FALSE(Data::LoadFromFile(file.path()));
}
