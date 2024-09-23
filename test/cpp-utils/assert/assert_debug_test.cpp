#include "cpp-utils/assert/AssertFailed.h"
#include <gtest/gtest.h>

#ifdef NDEBUG
#define REAL_NDEBUG_
#endif

//Include the ASSERT macro for a debug build
#undef NDEBUG
#include "cpp-utils/assert/assert.h"


TEST(AssertTest_DebugBuild, DoesntDieIfTrue) {
    ASSERT(true, "bla");
}

TEST(AssertTest_DebugBuild, DiesIfFalse) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_DEATH(
      ASSERT(false, "bla"),
      ""
    );
}

TEST(AssertTest_DebugBuild, whenDisablingAbort_thenThrowsIfFalse) {
    const cpputils::_assert::DisableAbortOnFailedAssertionRAII _disableAbort;
    EXPECT_THROW(
        ASSERT(false, "bla"),
        cpputils::AssertFailed
    );
}

TEST(AssertTest_DebugBuild, AssertMessage) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
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

#if !(defined(_MSC_VER) && defined(REAL_NDEBUG_))
TEST(AssertTest_DebugBuild, AssertMessageContainsBacktrace) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_DEATH(
        ASSERT(2==5, "my message"),
        "cpputils::"
    );
}
#else
TEST(AssertTest_DebugBuild, AssertMessageContainsBacktrace) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_DEATH(
        ASSERT(2==5, "my message"),
        "#1"
    );
}
#endif
