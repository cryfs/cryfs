#include "gtest/gtest.h"

#include <blockstore/utils/Key.h>

using ::testing::Test;

using std::string;

using namespace blockstore;

class KeyTest: public Test {};

TEST_F(KeyTest, CanGenerateRandomKeysWithoutCrashing) {
  Key result = Key::CreateRandomKey();
}

TEST_F(KeyTest, CreatedRandomKeysHaveCorrectLength) {
  auto key = Key::CreateRandomKey();
  EXPECT_EQ(Key::KEYLENGTH_STRING, key.AsString().size());
}
