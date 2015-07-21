#include <google/gtest/gtest.h>
#include <google/gmock/gmock.h>

//Include the fspp_assert macro for a debug build
#undef NDEBUG
#include "../../utils/assert.h"

using testing::MatchesRegex;

TEST(AssertTest_DebugBuild, DoesntDieIfTrue) {
    fspp_assert(true, "bla");
}

TEST(AssertTest_DebugBuild, DiesIfFalse) {
    EXPECT_DEATH(
      fspp_assert(false, "bla"),
      ""
    );
}

TEST(AssertTest_DebugBuild, AssertMessage) {
    EXPECT_DEATH(
      fspp_assert(2==5, "my message"),
      "Assertion \\[2==5\\] failed in .*/assert_debug_test.cpp:25: my message"
    );
}