#include <gmock/gmock.h>
#include "cpp-utils/assert/backtrace.h"

using std::string;
using testing::HasSubstr;

TEST(BacktraceTest, ContainsExecutableName) {
    string backtrace = cpputils::backtrace();
    EXPECT_THAT(backtrace, HasSubstr("cpp-utils-test"));
}

TEST(BacktraceTest, ContainsTopLevelLine) {
    string backtrace = cpputils::backtrace();
    EXPECT_THAT(backtrace, HasSubstr("BacktraceTest"));
    EXPECT_THAT(backtrace, HasSubstr("ContainsTopLevelLine"));
}
