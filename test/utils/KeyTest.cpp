#include "../testutils/DataBlockFixture.h"
#include "../../utils/Data.h"
#include "../../utils/Key.h"
#include "google/gtest/gtest.h"


using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::string;

using namespace blockstore;

class KeyTest: public Test {
public:
  const string KEY1_AS_STRING = "1491BB4932A389EE14BC7090AC772972";
  const string KEY2_AS_STRING = "272EE5517627CFA147A971A8E6E747E0";

  const DataBlockFixture KEY3_AS_BINARY;
  const DataBlockFixture KEY4_AS_BINARY;

  KeyTest() : KEY3_AS_BINARY(Key::KEYLENGTH_BINARY, 1), KEY4_AS_BINARY(Key::KEYLENGTH_BINARY, 2) {}

  void EXPECT_DATA_EQ(const DataBlockFixture &expected, const Data &actual) {
    EXPECT_EQ(expected.size(), actual.size());
    EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), expected.size()));
  }
};

TEST_F(KeyTest, CanGenerateRandomKeysWithoutCrashing) {
  Key result = Key::CreateRandomKey();
}

TEST_F(KeyTest, CreatedRandomKeysHaveCorrectLength) {
  Key key = Key::CreateRandomKey();
  EXPECT_EQ(Key::KEYLENGTH_STRING, key.ToString().size());
}

TEST_F(KeyTest, EqualsTrue) {
  Key key1_1 = Key::FromString(KEY1_AS_STRING);
  Key key1_2 = Key::FromString(KEY1_AS_STRING);

  EXPECT_TRUE(key1_1 == key1_2);
  EXPECT_TRUE(key1_2 == key1_1);
}

TEST_F(KeyTest, EqualsFalse) {
  Key key1_1 = Key::FromString(KEY1_AS_STRING);
  Key key2_1 = Key::FromString(KEY2_AS_STRING);

  EXPECT_FALSE(key1_1 == key2_1);
  EXPECT_FALSE(key2_1 == key1_1);
}

TEST_F(KeyTest, NotEqualsFalse) {
  Key key1_1 = Key::FromString(KEY1_AS_STRING);
  Key key1_2 = Key::FromString(KEY1_AS_STRING);

  EXPECT_FALSE(key1_1 != key1_2);
  EXPECT_FALSE(key1_2 != key1_1);
}

TEST_F(KeyTest, NotEqualsTrue) {
  Key key1_1 = Key::FromString(KEY1_AS_STRING);
  Key key2_1 = Key::FromString(KEY2_AS_STRING);

  EXPECT_TRUE(key1_1 != key2_1);
  EXPECT_TRUE(key2_1 != key1_1);
}

class KeyTestWithStringKeyParam: public KeyTest, public WithParamInterface<string> {};
INSTANTIATE_TEST_CASE_P(KeyTestWithStringKeyParam, KeyTestWithStringKeyParam, Values("2898B4B8A13CA63CBE0F0278CCE465DB", "6FFEBAD90C0DAA2B79628F0627CE9841"));

TEST_P(KeyTestWithStringKeyParam, FromAndToString) {
  Key key = Key::FromString(GetParam());
  EXPECT_EQ(GetParam(), key.ToString());
}

TEST_P(KeyTestWithStringKeyParam, ToAndFromString) {
  Key key = Key::FromString(GetParam());
  Key key2 = Key::FromString(key.ToString());
  EXPECT_EQ(key, key2);
}

class KeyTestWithBinaryKeyParam: public KeyTest, public WithParamInterface<const DataBlockFixture*> {
public:
  static const DataBlockFixture VALUE1;
  static const DataBlockFixture VALUE2;
};
const DataBlockFixture KeyTestWithBinaryKeyParam::VALUE1(Key::KEYLENGTH_BINARY, 3);
const DataBlockFixture KeyTestWithBinaryKeyParam::VALUE2(Key::KEYLENGTH_BINARY, 4);
INSTANTIATE_TEST_CASE_P(KeyTestWithBinaryKeyParam, KeyTestWithBinaryKeyParam, Values(&KeyTestWithBinaryKeyParam::VALUE1, &KeyTestWithBinaryKeyParam::VALUE2));

TEST_P(KeyTestWithBinaryKeyParam, FromAndToBinary) {
  Key key = Key::FromBinary((uint8_t*)GetParam()->data());
  Data keydata(Key::KEYLENGTH_BINARY);
  key.ToBinary(keydata.data());
  EXPECT_DATA_EQ(*GetParam(), keydata);
}

TEST_P(KeyTestWithBinaryKeyParam, ToAndFromBinary) {
  Key key = Key::FromBinary((uint8_t*)GetParam()->data());
  Data stored(Key::KEYLENGTH_BINARY);
  key.ToBinary(stored.data());
  Key loaded = Key::FromBinary(stored.data());
  EXPECT_EQ(key, loaded);
}

class KeyTestWithKeyParam: public KeyTest, public WithParamInterface<Key> {};
INSTANTIATE_TEST_CASE_P(KeyTestWithKeyParam, KeyTestWithKeyParam, Values(Key::FromString("2898B4B8A13CA63CBE0F0278CCE465DB"), Key::FromString("6FFEBAD90C0DAA2B79628F0627CE9841")));

TEST_P(KeyTestWithKeyParam, CopyConstructor) {
  Key copy(GetParam());
  EXPECT_EQ(GetParam(), copy);
}

TEST_F(KeyTest, CopyConstructorDoesntChangeSource) {
  Key key1 = Key::FromString(KEY1_AS_STRING);
  Key key2(key1);
  EXPECT_EQ(KEY1_AS_STRING, key1.ToString());
}

TEST_P(KeyTestWithKeyParam, IsEqualAfterAssignment1) {
  Key key2 = Key::FromString(KEY2_AS_STRING);
  EXPECT_NE(GetParam(), key2);
  key2 = GetParam();
  EXPECT_EQ(GetParam(), key2);
}

TEST_F(KeyTest, AssignmentDoesntChangeSource) {
  Key key1 = Key::FromString(KEY1_AS_STRING);
  Key key2 = Key::FromString(KEY2_AS_STRING);
  key2 = key1;
  EXPECT_EQ(KEY1_AS_STRING, key1.ToString());
}

// This tests that a Key object is very lightweight
// (we will often pass keys around)
TEST_F(KeyTest, KeyIsLightweightObject) {
  EXPECT_EQ(Key::KEYLENGTH_BINARY, sizeof(Key));
}
