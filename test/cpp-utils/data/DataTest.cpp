#include "cpp-utils/data/DataFixture.h"
#include "cpp-utils/data/Data.h"
#include "cpp-utils/data/SerializationHelper.h"
#include <gmock/gmock.h>
#include "cpp-utils/tempfile/TempFile.h"

#include <fstream>

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;
using ::testing::_;

using cpputils::TempFile;

using std::ifstream;
using std::ofstream;
using std::string;

namespace bf = boost::filesystem;

using namespace cpputils;

class DataTest: public Test {
public:
  bool DataIsZeroes(const Data &data) {
    for (size_t i = 0; i != data.size(); ++ i) {
      if (deserialize<uint8_t>(data.dataOffset(i)) != 0) {
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
    ofstream file(filepath.string().c_str(), std::ios::binary | std::ios::trunc);
    file.write(static_cast<const char*>(data.data()), data.size());
  }

  static void EXPECT_STORED_FILE_DATA_CORRECT(const Data &data, const bf::path &filepath) {
    EXPECT_EQ(data.size(), bf::file_size(filepath));

    ifstream file(filepath.string().c_str(), std::ios::binary);
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

TEST_F(DataTest, ChangingCopyDoesntChangeOriginal) {
  Data original = DataFixture::generate(1024);
  Data copy = original.copy();
  serialize<uint8_t>(copy.data(), deserialize<uint8_t>(copy.data()) + 1);
  EXPECT_EQ(DataFixture::generate(1024), original);
  EXPECT_NE(copy, original);
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

TEST_F(DataTest, MoveConstructor) {
  Data original = DataFixture::generate(1024);
  Data copy(std::move(original));
  EXPECT_EQ(DataFixture::generate(1024), copy);
  EXPECT_EQ(nullptr, original.data()); // NOLINT (intentional use-after-move)
  EXPECT_EQ(0u, original.size()); // NOLINT (intentional use-after-move)
}

TEST_F(DataTest, MoveAssignment) {
  Data original = DataFixture::generate(1024);
  Data copy(0);
  copy = std::move(original);
  EXPECT_EQ(DataFixture::generate(1024), copy);
  EXPECT_EQ(nullptr, original.data()); // NOLINT (intentional use-after-move)
  EXPECT_EQ(0u, original.size()); // NOLINT (intentional use-after-move)
}

TEST_F(DataTest, Equality) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(1024);
  EXPECT_TRUE(data1 == data2);
  EXPECT_FALSE(data1 != data2);
}

TEST_F(DataTest, Inequality_DifferentSize) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(1023);
  EXPECT_FALSE(data1 == data2);
  EXPECT_TRUE(data1 != data2);
}

TEST_F(DataTest, Inequality_DifferentFirstByte) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(1024);
  serialize<uint8_t>(data2.data(), deserialize<uint8_t>(data2.data()) + 1);
  EXPECT_FALSE(data1 == data2);
  EXPECT_TRUE(data1 != data2);
}

TEST_F(DataTest, Inequality_DifferentMiddleByte) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(1024);
  serialize<uint8_t>(data2.dataOffset(500), deserialize<uint8_t>(data2.dataOffset(500)) + 1);
  EXPECT_FALSE(data1 == data2);
  EXPECT_TRUE(data1 != data2);
}

TEST_F(DataTest, Inequality_DifferentLastByte) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(1024);
  serialize<uint8_t>(data2.dataOffset(1023), deserialize<uint8_t>(data2.dataOffset(1023)) + 1);
  EXPECT_FALSE(data1 == data2);
  EXPECT_TRUE(data1 != data2);
}

#ifdef __x86_64__
TEST_F(DataTest, LargesizeSize) {
  //Needs 64bit for representation. This value isn't in the size param list, because the list is also used for read/write checks.
  uint64_t size = static_cast<uint64_t>(4.5L*1024*1024*1024);
  Data data(size);
  EXPECT_EQ(size, data.size());
}
#else
#if defined(_MSC_VER)
#pragma message This is not a 64bit architecture. Large size data tests are disabled.
#else
#warning This is not a 64bit architecture. Large size data tests are disabled.
#endif
#endif

TEST_F(DataTest, LoadingNonexistingFile) {
  TempFile file(false); // Pass false to constructor, so the tempfile is not created
  EXPECT_FALSE(Data::LoadFromFile(file.path()));
}

class DataTestWithStringParam: public DataTest, public WithParamInterface<string> {};
INSTANTIATE_TEST_CASE_P(DataTestWithStringParam, DataTestWithStringParam, Values("", "2898B4B8A13C0F0278CCE465DB", "6FFEBAD90C0DAA2B79628F0627CE9841"));

TEST_P(DataTestWithStringParam, FromAndToString) {
  Data data = Data::FromString(GetParam());
  EXPECT_EQ(GetParam(), data.ToString());
}

TEST_P(DataTestWithStringParam, ToAndFromString) {
  Data data = Data::FromString(GetParam());
  Data data2 = Data::FromString(data.ToString());
  EXPECT_EQ(data, data2);
}

struct MockAllocator final : public Allocator {
    MOCK_METHOD1(allocate, void* (size_t));
    MOCK_METHOD2(free, void(void*, size_t));
};

class DataTestWithMockAllocator: public DataTest {
public:
    char ptr_target{};

    unique_ref<MockAllocator> allocator = make_unique_ref<MockAllocator>();
    MockAllocator* allocator_ptr = allocator.get();
};

TEST_F(DataTestWithMockAllocator, whenCreatingNewData_thenTakesItFromAllocator) {
  EXPECT_CALL(*allocator, allocate(5)).Times(1).WillOnce(Return(&ptr_target));
  Data data(5, std::move(allocator));

  EXPECT_EQ(&ptr_target, data.data());
}

TEST_F(DataTestWithMockAllocator, whenDestructingData_thenFreesItInAllocator) {
    EXPECT_CALL(*allocator, allocate(5)).Times(1).WillOnce(Return(&ptr_target));
    Data data(5, std::move(allocator));

    EXPECT_CALL(*allocator_ptr, free(&ptr_target, 5)).Times(1);
}

TEST_F(DataTestWithMockAllocator, whenMoveConstructing_thenOnlyFreesOnce) {
    EXPECT_CALL(*allocator, allocate(5)).Times(1).WillOnce(Return(&ptr_target));

    Data data(5, std::move(allocator));
    Data data2 = std::move(data);

    EXPECT_CALL(*allocator_ptr, free(&ptr_target, 5)).Times(1);
}

TEST_F(DataTestWithMockAllocator, whenMoveAssigning_thenOnlyFreesOnce) {
    EXPECT_CALL(*allocator, allocate(5)).Times(1).WillOnce(Return(&ptr_target));

    Data data(5, std::move(allocator));
    Data data2(3);
    data2 = std::move(data);

    EXPECT_CALL(*allocator_ptr, free(&ptr_target, 5)).Times(1);
}

TEST_F(DataTestWithMockAllocator, whenMoveConstructing_thenOnlyFreesWhenSecondIsDestructed) {
    EXPECT_CALL(*allocator, allocate(5)).Times(1).WillOnce(Return(&ptr_target));
    EXPECT_CALL(*allocator_ptr, free(_, _)).Times(0);

    auto data = std::make_unique<Data>(5, std::move(allocator));
    Data data2 = std::move(*data);
    data.reset();

    EXPECT_CALL(*allocator_ptr, free(&ptr_target, 5)).Times(1);
}

TEST_F(DataTestWithMockAllocator, whenMoveAssigning_thenOnlyFreesWhenSecondIsDestructed) {
    EXPECT_CALL(*allocator, allocate(5)).Times(1).WillOnce(Return(&ptr_target));
    EXPECT_CALL(*allocator_ptr, free(_, _)).Times(0);

    auto data = std::make_unique<Data>(5, std::move(allocator));
    Data data2(3);
    data2 = std::move(*data);
    data.reset();

    EXPECT_CALL(*allocator_ptr, free(&ptr_target, 5)).Times(1);
}
