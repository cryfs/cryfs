#include <gtest/gtest.h>
#include "blobstore/implementations/onblocks/utils/Math.h"

#include <limits>

using namespace blobstore::onblocks::utils;
using ::testing::Test;
using std::numeric_limits;

class MaxZeroSubtractionTest: public Test {};

TEST_F(MaxZeroSubtractionTest, SubtractToZero1) {
  EXPECT_EQ(0, maxZeroSubtraction(0, 0));
}

TEST_F(MaxZeroSubtractionTest, SubtractToZero2) {
  EXPECT_EQ(0, maxZeroSubtraction(5, 5));
}

TEST_F(MaxZeroSubtractionTest, SubtractToZero3) {
  EXPECT_EQ(0, maxZeroSubtraction(184930, 184930));
}

TEST_F(MaxZeroSubtractionTest, SubtractToZero4) {
  EXPECT_EQ(0u, maxZeroSubtraction(numeric_limits<uint32_t>::max()-1, numeric_limits<uint32_t>::max()-1));
}

TEST_F(MaxZeroSubtractionTest, SubtractToZero5) {
  EXPECT_EQ(0u, maxZeroSubtraction(numeric_limits<uint32_t>::max(), numeric_limits<uint32_t>::max()));
}

TEST_F(MaxZeroSubtractionTest, SubtractPositive1) {
  EXPECT_EQ(1, maxZeroSubtraction(5, 4));
}

TEST_F(MaxZeroSubtractionTest, SubtractPositive2) {
  EXPECT_EQ(181081, maxZeroSubtraction(184930, 3849));
}

TEST_F(MaxZeroSubtractionTest, SubtractPositive3) {
  EXPECT_EQ(numeric_limits<uint32_t>::max()-1, maxZeroSubtraction(numeric_limits<uint32_t>::max(), UINT32_C(1)));
}

TEST_F(MaxZeroSubtractionTest, SubtractPositive4) {
  EXPECT_EQ(5u, maxZeroSubtraction(numeric_limits<uint32_t>::max(), numeric_limits<uint32_t>::max()-5));
}

TEST_F(MaxZeroSubtractionTest, SubtractNegative1) {
  EXPECT_EQ(0, maxZeroSubtraction(4, 5));
}

TEST_F(MaxZeroSubtractionTest, SubtractNegative2) {
  EXPECT_EQ(0, maxZeroSubtraction(3849, 184930));
}

TEST_F(MaxZeroSubtractionTest, SubtractNegative3) {
  EXPECT_EQ(0u, maxZeroSubtraction(numeric_limits<uint32_t>::max()-1, numeric_limits<uint32_t>::max()));
}

TEST_F(MaxZeroSubtractionTest, SubtractNegative4) {
  EXPECT_EQ(0u, maxZeroSubtraction(numeric_limits<uint32_t>::max()-5, numeric_limits<uint32_t>::max()));
}

TEST_F(MaxZeroSubtractionTest, SubtractNegative5) {
  EXPECT_EQ(0u, maxZeroSubtraction(UINT32_C(5), numeric_limits<uint32_t>::max()));
}

TEST_F(MaxZeroSubtractionTest, SubtractFromZero1) {
  EXPECT_EQ(0, maxZeroSubtraction(0, 1));
}

TEST_F(MaxZeroSubtractionTest, SubtractFromZero2) {
  EXPECT_EQ(0, maxZeroSubtraction(0, 184930));
}

TEST_F(MaxZeroSubtractionTest, SubtractFromZero3) {
  EXPECT_EQ(0u, maxZeroSubtraction(UINT32_C(0), numeric_limits<uint32_t>::max()));
}

TEST_F(MaxZeroSubtractionTest, 64bit_valid) {
  uint64_t value = UINT64_C(1024)*1024*1024*1024;
  EXPECT_GT(value, std::numeric_limits<uint32_t>::max());
  EXPECT_EQ(value*1024-value, maxZeroSubtraction(value*1024, value));
}

TEST_F(MaxZeroSubtractionTest, 64bit_zero) {
  uint64_t value = UINT64_C(1024)*1024*1024*1024;
  EXPECT_GT(value, std::numeric_limits<uint32_t>::max());
  EXPECT_EQ(0u, maxZeroSubtraction(value, value*1024));
}
