#include "gtest/gtest.h"

#include <blockstore/utils/Key.h>
#include "blockstore/utils/Data.h"
#include "test/testutils/DataBlockFixture.h"

using ::testing::Test;

using std::string;

using namespace blockstore;

class KeyTest: public Test {
public:
  //TODO Use parametrized tests
  const string KEY1_AS_STRING = "1491BB4932A389EE14BC7090AC772972";
  const string KEY2_AS_STRING = "272EE5517627CFA147A971A8E6E747E0";

  const DataBlockFixture KEY3_AS_BINARY;
  const DataBlockFixture KEY4_AS_BINARY;

  KeyTest() : KEY3_AS_BINARY(Key::KEYLENGTH_BINARY, 1), KEY4_AS_BINARY(Key::KEYLENGTH_BINARY, 2) {}
};

#define EXPECT_DATA_EQ(expected, actual) {                                       \
  EXPECT_EQ(expected.size(), actual.size());                                     \
  EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), expected.size()));    \
}                                                                                \

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

TEST_F(KeyTest, FromAndToString1) {
  Key key = Key::FromString(KEY1_AS_STRING);
  EXPECT_EQ(KEY1_AS_STRING, key.ToString());
}

TEST_F(KeyTest, FromAndToString2) {
  Key key = Key::FromString(KEY2_AS_STRING);
  EXPECT_EQ(KEY2_AS_STRING, key.ToString());
}

TEST_F(KeyTest, ToAndFromString1) {
  Key key = Key::FromString(KEY1_AS_STRING);
  Key key2 = Key::FromString(key.ToString());
  EXPECT_EQ(key, key2);
}

TEST_F(KeyTest, ToAndFromString2) {
  Key key = Key::FromString(KEY2_AS_STRING);
  Key key2 = Key::FromString(key.ToString());
  EXPECT_EQ(key, key2);
}

TEST_F(KeyTest, FromAndToBinary1) {
  Key key = Key::FromBinary((uint8_t*)KEY3_AS_BINARY.data());
  Data keydata(Key::KEYLENGTH_BINARY);
  key.ToBinary(keydata.data());
  EXPECT_DATA_EQ(KEY3_AS_BINARY, keydata);
}

TEST_F(KeyTest, FromAndToBinary2) {
  Key key = Key::FromBinary((uint8_t*)KEY4_AS_BINARY.data());
  Data keydata(Key::KEYLENGTH_BINARY);
  key.ToBinary(keydata.data());
  EXPECT_DATA_EQ(KEY4_AS_BINARY, keydata);
}

TEST_F(KeyTest, ToAndFromBinary1) {
  Key key = Key::FromBinary((uint8_t*)KEY3_AS_BINARY.data());
  Data stored(Key::KEYLENGTH_BINARY);
  key.ToBinary(stored.data());
  Key loaded = Key::FromBinary(stored.data());
  EXPECT_EQ(key, loaded);
}

TEST_F(KeyTest, ToAndFromBinary2) {
  Key key = Key::FromBinary((uint8_t*)KEY4_AS_BINARY.data());
  Data stored(Key::KEYLENGTH_BINARY);
  key.ToBinary(stored.data());
  Key loaded = Key::FromBinary(stored.data());
  EXPECT_EQ(key, loaded);
}

TEST_F(KeyTest, CopyConstructor1) {
  Key key = Key::FromString(KEY1_AS_STRING);
  Key copy(key);
  EXPECT_EQ(key, copy);
}

TEST_F(KeyTest, CopyConstructor2) {
  Key key = Key::FromString(KEY2_AS_STRING);
  Key copy(key);
  EXPECT_EQ(key, copy);
}

TEST_F(KeyTest, CopyConstructorDoesntChangeSource) {
  Key key1 = Key::FromString(KEY1_AS_STRING);
  Key key2(key1);
  EXPECT_EQ(KEY1_AS_STRING, key1.ToString());
}

TEST_F(KeyTest, IsEqualAfterAssignment1) {
  Key key1 = Key::FromString(KEY1_AS_STRING);
  Key key2 = Key::FromString(KEY2_AS_STRING);
  EXPECT_NE(key1, key2);
  key2 = key1;
  EXPECT_EQ(key1, key2);
}

TEST_F(KeyTest, IsEqualAfterAssignment2) {
  Key key1 = Key::FromString(KEY2_AS_STRING);
  Key key2 = Key::FromString(KEY1_AS_STRING);
  EXPECT_NE(key1, key2);
  key2 = key1;
  EXPECT_EQ(key1, key2);
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
