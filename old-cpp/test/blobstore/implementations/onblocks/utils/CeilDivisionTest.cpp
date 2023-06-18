#include <gtest/gtest.h>
#include "blobstore/implementations/onblocks/utils/Math.h"

#include <limits>

using namespace blobstore::onblocks::utils;
using ::testing::Test;
using std::numeric_limits;

class CeilDivisionTest: public Test {};

TEST_F(CeilDivisionTest, Divide0_4) {
 EXPECT_EQ(0, ceilDivision(0, 4));
}

TEST_F(CeilDivisionTest, Divide1_4) {
 EXPECT_EQ(1, ceilDivision(1, 4));
}

TEST_F(CeilDivisionTest, Divide2_4) {
 EXPECT_EQ(1, ceilDivision(2, 4));
}

TEST_F(CeilDivisionTest, Divide3_4) {
 EXPECT_EQ(1, ceilDivision(3, 4));
}

TEST_F(CeilDivisionTest, Divide4_4) {
 EXPECT_EQ(1, ceilDivision(4, 4));
}

TEST_F(CeilDivisionTest, Divide5_4) {
 EXPECT_EQ(2, ceilDivision(5, 4));
}

TEST_F(CeilDivisionTest, Divide6_4) {
 EXPECT_EQ(2, ceilDivision(6, 4));
}

TEST_F(CeilDivisionTest, Divide7_4) {
 EXPECT_EQ(2, ceilDivision(7, 4));
}

TEST_F(CeilDivisionTest, Divide8_4) {
 EXPECT_EQ(2, ceilDivision(8, 4));
}

TEST_F(CeilDivisionTest, Divide9_4) {
 EXPECT_EQ(3, ceilDivision(9, 4));
}

TEST_F(CeilDivisionTest, Divide0_1) {
  EXPECT_EQ(0, ceilDivision(0, 1));
}

TEST_F(CeilDivisionTest, Divide5_1) {
  EXPECT_EQ(5, ceilDivision(5, 1));
}

TEST_F(CeilDivisionTest, Divide7_2) {
  EXPECT_EQ(4, ceilDivision(7, 2));
}

TEST_F(CeilDivisionTest, Divide8_2) {
  EXPECT_EQ(4, ceilDivision(8, 2));
}

TEST_F(CeilDivisionTest, Divide9_2) {
  EXPECT_EQ(5, ceilDivision(9, 2));
}

TEST_F(CeilDivisionTest, Divide1_1) {
  EXPECT_EQ(1, ceilDivision(1, 1));
}

TEST_F(CeilDivisionTest, Divide5_5) {
  EXPECT_EQ(1, ceilDivision(5, 5));
}

TEST_F(CeilDivisionTest, DivideLargeByItself) {
  EXPECT_EQ(1, ceilDivision(183495303, 183495303));
}

TEST_F(CeilDivisionTest, 64bit) {
  uint64_t base = UINT64_C(1024)*1024*1024*1024;
  EXPECT_GT(base, std::numeric_limits<uint32_t>::max());
  EXPECT_EQ(base/1024, ceilDivision(base, UINT64_C(1024)));
}
