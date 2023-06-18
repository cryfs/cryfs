#include <gtest/gtest.h>

#include "cpp-utils/data/Data.h"
#include "cpp-utils/data/DataFixture.h"

using ::testing::Test;


using namespace cpputils;

class DataFixtureTest: public Test {
};

TEST_F(DataFixtureTest, CreateEmptyFixture) {
  Data data = DataFixture::generate(0);
  EXPECT_EQ(0u, data.size());
}

TEST_F(DataFixtureTest, CreateOneByteFixture) {
  Data data = DataFixture::generate(1);
  EXPECT_EQ(1u, data.size());
}

TEST_F(DataFixtureTest, CreateLargerFixture) {
  Data data = DataFixture::generate(20 * 1024 * 1024);
  EXPECT_EQ(20u * 1024u * 1024u, data.size());
}

TEST_F(DataFixtureTest, FixturesAreDeterministic_DefaultSeed) {
  Data data1 = DataFixture::generate(1024 * 1024);
  Data data2 = DataFixture::generate(1024 * 1024);
  EXPECT_EQ(data1, data2);
}

TEST_F(DataFixtureTest, FixturesAreDeterministic_SeedIs5) {
  Data data1 = DataFixture::generate(1024 * 1024, 5);
  Data data2 = DataFixture::generate(1024 * 1024, 5);
  EXPECT_EQ(data1, data2);
}

TEST_F(DataFixtureTest, DifferentSeedIsDifferentFixture) {
  Data data1 = DataFixture::generate(1024 * 1024, 0);
  Data data2 = DataFixture::generate(1024 * 1024, 1);
  EXPECT_NE(data1, data2);
}

TEST_F(DataFixtureTest, FixturesAreDeterministic_DifferentSize_DefaultSeed_1) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(1);

  EXPECT_EQ(0, std::memcmp(data1.data(), data2.data(), 1));
}

TEST_F(DataFixtureTest, FixturesAreDeterministic_DifferentSize_DefaultSeed_2) {
  Data data1 = DataFixture::generate(1024);
  Data data2 = DataFixture::generate(501);  //Intentionally not 64bit-aligned, because the generate() function generates 64bit values for performance

  EXPECT_EQ(0, std::memcmp(data1.data(), data2.data(), 501));
}

TEST_F(DataFixtureTest, FixturesAreDeterministic_DifferentSize_SeedIs5_1) {
  Data data1 = DataFixture::generate(1024, 5);
  Data data2 = DataFixture::generate(1, 5);

  EXPECT_EQ(0, std::memcmp(data1.data(), data2.data(), 1));
}

TEST_F(DataFixtureTest, FixturesAreDeterministic_DifferentSize_SeedIs5_2) {
  Data data1 = DataFixture::generate(1024, 5);
  Data data2 = DataFixture::generate(501, 5);  //Intentionally not 64bit-aligned, because the generate() function generates 64bit values for performance

  EXPECT_EQ(0, std::memcmp(data1.data(), data2.data(), 501));
}
