#include <gtest/gtest.h>
#include <gmock/gmock.h>

//Include the ASSERT macro for a release build
#ifndef NDEBUG
#define NDEBUG
#endif
#include "cpp-utils/assert/assert.h"

using testing::MatchesRegex;
using testing::HasSubstr;

TEST(AssertTest_ReleaseBuild, DoesntThrowIfTrue) {
  ASSERT(true, "bla");
}

TEST(AssertTest_ReleaseBuild, ThrowsIfFalse) {
  EXPECT_THROW(
    ASSERT(false, "bla"),
    cpputils::AssertFailed
  );
}

TEST(AssertTest_ReleaseBuild, AssertMessage) {
  try {
    ASSERT(2==5, "my message");
    FAIL();
  } catch (const cpputils::AssertFailed &e) {
    EXPECT_THAT(e.what(), MatchesRegex(
        "Assertion \\[2==5\\] failed in .*/assert_release_test.cpp:26: my message.*"
    ));
  }
}

TEST(AssertTest_ReleaseBuild, AssertMessageContainsBacktrace) {
  try {
    ASSERT(2==5, "my message");
    FAIL();
  } catch (const cpputils::AssertFailed &e) {
    EXPECT_THAT(e.what(), HasSubstr(
            "cpputils::backtrace"
    ));
  }
}
