#include "gtest/gtest.h"

#include <blockstore/utils/Key.h>

using ::testing::Test;

using std::string;

using namespace blockstore;

class KeyTest: public Test {
public:
  const string KEY1_DATA = "1491BB4932A389EE14BC7090AC772972";
  const string KEY2_DATA = "272EE5517627CFA147A971A8E6E747E0";
};

TEST_F(KeyTest, CanGenerateRandomKeysWithoutCrashing) {
  Key result = Key::CreateRandomKey();
}

TEST_F(KeyTest, CreatedRandomKeysHaveCorrectLength) {
  auto key = Key::CreateRandomKey();
  EXPECT_EQ(Key::KEYLENGTH_STRING, key.AsString().size());
}

TEST_F(KeyTest, EqualsTrue) {
  auto key1_1 = Key::FromString(KEY1_DATA);
  auto key1_2 = Key::FromString(KEY1_DATA);

  EXPECT_TRUE(key1_1 == key1_2);
  EXPECT_TRUE(key1_2 == key1_1);
}

TEST_F(KeyTest, EqualsFalse) {
  auto key1_1 = Key::FromString(KEY1_DATA);
  auto key2_1 = Key::FromString(KEY2_DATA);

  EXPECT_FALSE(key1_1 == key2_1);
  EXPECT_FALSE(key2_1 == key1_1);
}

TEST_F(KeyTest, NotEqualsFalse) {
  auto key1_1 = Key::FromString(KEY1_DATA);
  auto key1_2 = Key::FromString(KEY1_DATA);

  EXPECT_FALSE(key1_1 != key1_2);
  EXPECT_FALSE(key1_2 != key1_1);
}

TEST_F(KeyTest, NotEqualsTrue) {
  auto key1_1 = Key::FromString(KEY1_DATA);
  auto key2_1 = Key::FromString(KEY2_DATA);

  EXPECT_TRUE(key1_1 != key2_1);
  EXPECT_TRUE(key2_1 != key1_1);
}

TEST_F(KeyTest, FromAndToString1) {
  auto key = Key::FromString(KEY1_DATA);
  EXPECT_EQ(KEY1_DATA, key.AsString());
}

TEST_F(KeyTest, FromAndToString2) {
  auto key = Key::FromString(KEY2_DATA);
  EXPECT_EQ(KEY2_DATA, key.AsString());
}

TEST_F(KeyTest, ToAndFromString1) {
  auto key = Key::FromString(KEY1_DATA);
  auto key2 = Key::FromString(key.AsString());
  EXPECT_EQ(key, key2);
}

TEST_F(KeyTest, ToAndFromString2) {
  auto key = Key::FromString(KEY2_DATA);
  auto key2 = Key::FromString(key.AsString());
  EXPECT_EQ(key, key2);
}
