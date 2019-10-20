#include <gtest/gtest.h>
#include "blobstore/implementations/onblocks/utils/Math.h"

using namespace blobstore::onblocks::utils;
using ::testing::Test;

class IntPowTest: public Test {};

TEST_F(IntPowTest, ExponentAndBaseAreZero) {
  EXPECT_EQ(1, intPow(0, 0));
}

TEST_F(IntPowTest, ExponentIsZero1) {
  EXPECT_EQ(1, intPow(1, 0));
}

TEST_F(IntPowTest, ExponentIsZero2) {
  EXPECT_EQ(1, intPow(1000, 0));
}

TEST_F(IntPowTest, BaseIsZero1) {
  EXPECT_EQ(0, intPow(0, 1));
}

TEST_F(IntPowTest, BaseIsZero2) {
  EXPECT_EQ(0, intPow(0, 1000));
}

TEST_F(IntPowTest, ExponentIsOne1) {
  EXPECT_EQ(0, intPow(0, 1));
}

TEST_F(IntPowTest, ExponentIsOne2) {
  EXPECT_EQ(2, intPow(2, 1));
}

TEST_F(IntPowTest, ExponentIsOne3) {
  EXPECT_EQ(1000, intPow(1000, 1));
}

TEST_F(IntPowTest, BaseIsTwo1) {
  EXPECT_EQ(1024, intPow(2, 10));
}

TEST_F(IntPowTest, BaseIsTwo2) {
  EXPECT_EQ(1024*1024, intPow(2, 20));
}

TEST_F(IntPowTest, BaseIsTwo3) {
  EXPECT_EQ(1024*1024*1024, intPow(2, 30));
}

TEST_F(IntPowTest, BaseIsTen1) {
  EXPECT_EQ(100, intPow(10, 2));
}

TEST_F(IntPowTest, BaseIsTen2) {
  EXPECT_EQ(1000000, intPow(10, 6));
}

TEST_F(IntPowTest, ArbitraryNumbers1) {
  EXPECT_EQ(4096, intPow(4, 6));
}

TEST_F(IntPowTest, ArbitraryNumbers2) {
  EXPECT_EQ(1296, intPow(6, 4));
}

TEST_F(IntPowTest, ArbitraryNumbers3) {
  EXPECT_EQ(282475249, intPow(7, 10));
}

TEST_F(IntPowTest, 64bit) {
  uint64_t value = UINT64_C(1024)*1024*1024*1024;
  EXPECT_GT(value, std::numeric_limits<uint32_t>::max());
  EXPECT_EQ(value*value*value, intPow(value, UINT64_C(3)));
}
