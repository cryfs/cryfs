#include "gtest/gtest.h"

#include "blobstore/utils/RandomKeyGenerator.h"

using ::testing::Test;

using std::string;

using namespace blobstore;

class RandomKeyGeneratorTest: public Test {};

TEST_F(RandomKeyGeneratorTest, RunsWithoutCrashes) {
  string result = RandomKeyGenerator::singleton().create();
}

TEST_F(RandomKeyGeneratorTest, KeySizeIsAsSpecified) {
  string result = RandomKeyGenerator::singleton().create();
  EXPECT_EQ(RandomKeyGenerator::KEYLENGTH, result.size());
}
