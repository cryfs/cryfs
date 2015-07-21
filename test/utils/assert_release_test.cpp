#include <google/gtest/gtest.h>
#include <google/gmock/gmock.h>

//Include the fspp_assert macro for a release build
#define NDEBUG
#include "../../utils/assert.h"

using testing::MatchesRegex;

TEST(AssertTest_ReleaseBuild, DoesntThrowIfTrue) {
  fspp_assert(true, "bla");
}

TEST(AssertTest_ReleaseBuild, ThrowsIfFalse) {
  EXPECT_THROW(
    fspp_assert(false, "bla"),
    fspp::IOException
  );
}

TEST(AssertTest_ReleaseBuild, AssertMessage) {
  try {
    fspp_assert(2==5, "my message");
    FAIL();
  } catch (const fspp::IOException &e) {
    EXPECT_THAT(e.what(), MatchesRegex(
        "Assertion \\[2==5\\] failed in .*/assert_release_test.cpp:23: my message"
    ));
  }
}