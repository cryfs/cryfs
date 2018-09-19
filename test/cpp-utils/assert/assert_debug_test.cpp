#include <gtest/gtest.h>
#include <gmock/gmock.h>

//Include the ASSERT macro for a debug build
#undef NDEBUG
#include "cpp-utils/assert/assert.h"


TEST(AssertTest_DebugBuild, DoesntDieIfTrue) {
    ASSERT(true, "bla");
}

TEST(AssertTest_DebugBuild, DiesIfFalse) {
    EXPECT_DEATH(
      ASSERT(false, "bla"),
      ""
    );
}

TEST(AssertTest_DebugBuild, AssertMessage) {
#if defined(_MSC_VER)
constexpr const char* EXPECTED = R"(Assertion \[2==5\] failed in .*assert_debug_test.cpp:\d+: my message)";
#else
constexpr const char* EXPECTED = R"(Assertion \[2==5\] failed in .*assert_debug_test.cpp:[0-9]+: my message)";
#endif
    EXPECT_DEATH(
      ASSERT(2==5, "my message"),
		EXPECTED
    );
}

TEST(AssertTest_DebugBuild, AssertMessageContainsBacktrace) {
    EXPECT_DEATH(
        ASSERT(2==5, "my message"),
        "cpputils::"
    );
}
