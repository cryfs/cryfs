#include "cpp-utils/data/DataFixture.h"
#include "cpp-utils/data/FixedSizeData.h"
#include "cpp-utils/data/Data.h"
#include <gtest/gtest.h>


using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::string;

using namespace cpputils;

class FixedSizeDataTest: public Test {
public:
  static constexpr size_t SIZE = 16;

  const string DATA1_AS_STRING = "1491BB4932A389EE14BC7090AC772972";
  const string DATA2_AS_STRING = "272EE5517627CFA147A971A8E6E747E0";

  const Data DATA3_AS_BINARY;
  const Data DATA4_AS_BINARY;

  FixedSizeDataTest() : DATA3_AS_BINARY(DataFixture::generate(SIZE, 1)), DATA4_AS_BINARY(DataFixture::generate(SIZE, 2)) {}

  template<size_t SIZE>
  void EXPECT_DATA_EQ(const Data &expected, const FixedSizeData<SIZE> &actual) {
    EXPECT_EQ(expected.size(), SIZE);
    EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), SIZE));
  }
};

constexpr size_t FixedSizeDataTest::SIZE;

TEST_F(FixedSizeDataTest, EqualsTrue) {
  FixedSizeData<SIZE> DATA1_1 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);
  FixedSizeData<SIZE> DATA1_2 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);

  EXPECT_TRUE(DATA1_1 == DATA1_2);
  EXPECT_TRUE(DATA1_2 == DATA1_1);
}

TEST_F(FixedSizeDataTest, EqualsFalse) {
  FixedSizeData<SIZE> DATA1_1 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);
  FixedSizeData<SIZE> DATA2_1 = FixedSizeData<SIZE>::FromString(DATA2_AS_STRING);

  EXPECT_FALSE(DATA1_1 == DATA2_1);
  EXPECT_FALSE(DATA2_1 == DATA1_1);
}

TEST_F(FixedSizeDataTest, NotEqualsFalse) {
  FixedSizeData<SIZE> DATA1_1 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);
  FixedSizeData<SIZE> DATA1_2 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);

  EXPECT_FALSE(DATA1_1 != DATA1_2);
  EXPECT_FALSE(DATA1_2 != DATA1_1);
}

TEST_F(FixedSizeDataTest, NotEqualsTrue) {
  FixedSizeData<SIZE> DATA1_1 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);
  FixedSizeData<SIZE> DATA2_1 = FixedSizeData<SIZE>::FromString(DATA2_AS_STRING);

  EXPECT_TRUE(DATA1_1 != DATA2_1);
  EXPECT_TRUE(DATA2_1 != DATA1_1);
}

class FixedSizeDataTestWithStringParam: public FixedSizeDataTest, public WithParamInterface<string> {};
INSTANTIATE_TEST_SUITE_P(FixedSizeDataTestWithStringParam, FixedSizeDataTestWithStringParam, Values("2898B4B8A13CA63CBE0F0278CCE465DB", "6FFEBAD90C0DAA2B79628F0627CE9841"));

TEST_P(FixedSizeDataTestWithStringParam, FromAndToString) {
  FixedSizeData<SIZE> data = FixedSizeData<SIZE>::FromString(GetParam());
  EXPECT_EQ(GetParam(), data.ToString());
}

TEST_P(FixedSizeDataTestWithStringParam, ToAndFromString) {
  FixedSizeData<SIZE> data = FixedSizeData<SIZE>::FromString(GetParam());
  FixedSizeData<SIZE> data2 = FixedSizeData<SIZE>::FromString(data.ToString());
  EXPECT_EQ(data, data2);
}

class FixedSizeDataTestWithBinaryParam: public FixedSizeDataTest, public WithParamInterface<const Data*> {
public:
  static const Data VALUE1;
  static const Data VALUE2;
};
const Data FixedSizeDataTestWithBinaryParam::VALUE1(DataFixture::generate(SIZE, 3));
const Data FixedSizeDataTestWithBinaryParam::VALUE2(DataFixture::generate(SIZE, 4));
INSTANTIATE_TEST_SUITE_P(FixedSizeDataTestWithBinaryParam, FixedSizeDataTestWithBinaryParam, Values(&FixedSizeDataTestWithBinaryParam::VALUE1, &FixedSizeDataTestWithBinaryParam::VALUE2));

TEST_P(FixedSizeDataTestWithBinaryParam, FromBinary) {
  FixedSizeData<SIZE> data = FixedSizeData<SIZE>::FromBinary(GetParam()->data());
  EXPECT_DATA_EQ(*GetParam(), data);
}

TEST_P(FixedSizeDataTestWithBinaryParam, FromAndToBinary) {
  FixedSizeData<SIZE> data = FixedSizeData<SIZE>::FromBinary(GetParam()->data());
  Data output(FixedSizeData<SIZE>::BINARY_LENGTH);
  data.ToBinary(output.data());
  EXPECT_EQ(*GetParam(), output);
}

TEST_P(FixedSizeDataTestWithBinaryParam, ToAndFromBinary) {
  FixedSizeData<SIZE> data = FixedSizeData<SIZE>::FromBinary(GetParam()->data());
  Data stored(FixedSizeData<SIZE>::BINARY_LENGTH);
  data.ToBinary(stored.data());
  FixedSizeData<SIZE> loaded = FixedSizeData<SIZE>::FromBinary(stored.data());
  EXPECT_EQ(data, loaded);
}

class FixedSizeDataTestWithParam: public FixedSizeDataTest, public WithParamInterface<FixedSizeData<FixedSizeDataTest::SIZE>> {};
INSTANTIATE_TEST_SUITE_P(FixedSizeDataTestWithParam, FixedSizeDataTestWithParam, Values(FixedSizeData<FixedSizeDataTest::SIZE>::FromString("2898B4B8A13CA63CBE0F0278CCE465DB"), FixedSizeData<FixedSizeDataTest::SIZE>::FromString("6FFEBAD90C0DAA2B79628F0627CE9841")));

TEST_P(FixedSizeDataTestWithParam, CopyConstructor) {
  FixedSizeData<SIZE> copy(GetParam());
  EXPECT_EQ(GetParam(), copy);
}

TEST_P(FixedSizeDataTestWithParam, Take_Half) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<SIZE/2> taken = source.take<SIZE/2>();
  EXPECT_EQ(0, std::memcmp(source.data(), taken.data(), SIZE/2));
}

TEST_P(FixedSizeDataTestWithParam, Drop_Half) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<SIZE/2> taken = source.drop<SIZE/2>();
  EXPECT_EQ(0, std::memcmp(source.data() + SIZE/2, taken.data(), SIZE/2));
}

TEST_P(FixedSizeDataTestWithParam, Take_One) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<1> taken = source.take<1>();
  EXPECT_EQ(0, std::memcmp(source.data(), taken.data(), 1));
}

TEST_P(FixedSizeDataTestWithParam, Drop_One) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<SIZE-1> taken = source.drop<1>();
  EXPECT_EQ(0, std::memcmp(source.data() + 1, taken.data(), SIZE-1));
}

TEST_P(FixedSizeDataTestWithParam, Take_Nothing) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<0> taken = source.take<0>();
  (void)taken; // silence unused variable warning
}

TEST_P(FixedSizeDataTestWithParam, Drop_Nothing) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<SIZE> taken = source.drop<0>();
  EXPECT_EQ(0, std::memcmp(source.data(), taken.data(), SIZE));
}

TEST_P(FixedSizeDataTestWithParam, Take_All) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<SIZE> taken = source.take<SIZE>();
  EXPECT_EQ(0, std::memcmp(source.data(), taken.data(), SIZE));
}

TEST_P(FixedSizeDataTestWithParam, Drop_All) {
  FixedSizeData<SIZE> source(GetParam());
  FixedSizeData<0> taken = source.drop<SIZE>();
  (void)taken; // silence unused variable warning
}

TEST_F(FixedSizeDataTest, CopyConstructorDoesntChangeSource) {
  FixedSizeData<SIZE> data1 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);
  FixedSizeData<SIZE> data2(data1);
  EXPECT_EQ(DATA1_AS_STRING, data1.ToString());
  (void)data2; // silence unused variable warning
}

TEST_P(FixedSizeDataTestWithParam, IsEqualAfterAssignment1) {
  FixedSizeData<SIZE> data2 = FixedSizeData<SIZE>::FromString(DATA2_AS_STRING);
  EXPECT_NE(GetParam(), data2);
  data2 = GetParam();
  EXPECT_EQ(GetParam(), data2);
}

TEST_F(FixedSizeDataTest, AssignmentDoesntChangeSource) {
  FixedSizeData<SIZE> data1 = FixedSizeData<SIZE>::FromString(DATA1_AS_STRING);
  FixedSizeData<SIZE> data2 = FixedSizeData<SIZE>::FromString(DATA2_AS_STRING);
  data2 = data1;
  EXPECT_EQ(DATA1_AS_STRING, data1.ToString());
}

// This tests that a FixedSizeData object is very lightweight
// (it is meant to be kept on stack and passed around)
TEST_F(FixedSizeDataTest, IsLightweightObject) {
  EXPECT_EQ(FixedSizeData<SIZE>::BINARY_LENGTH, sizeof(FixedSizeData<SIZE>));
}
