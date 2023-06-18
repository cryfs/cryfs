#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <regex>

#ifdef NDEBUG
#define REAL_NDEBUG_
#endif

//Include the ASSERT macro for a release build
#ifndef NDEBUG
#define NDEBUG 1
#endif
#include "cpp-utils/assert/assert.h"

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
	  std::string msg = e.what();
	  // For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	  /*EXPECT_THAT(e.what(), MatchesRegex(
		  R"(Assertion \[2==5\] failed in .*assert_release_test.cpp:27: my message)"
	  ));*/
	  EXPECT_TRUE(std::regex_search(e.what(), std::regex(R"(Assertion \[2==5\] failed in .*assert_release_test.cpp:30: my message)")));
  }
}

#if !(defined(_MSC_VER) && defined(REAL_NDEBUG_))
TEST(AssertTest_ReleaseBuild, AssertMessageContainsBacktrace) {
  try {
    ASSERT(2==5, "my message");
    FAIL();
  } catch (const cpputils::AssertFailed &e) {
    EXPECT_THAT(e.what(), HasSubstr(
            "cpputils::"
    ));
  }
}
#else
TEST(AssertTest_ReleaseBuild, AssertMessageContainsBacktrace) {
  try {
    ASSERT(2==5, "my message");
    FAIL();
  } catch (const cpputils::AssertFailed &e) {
    EXPECT_THAT(e.what(), HasSubstr(
      "#1"
    ));
  }
}
#endif
