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

namespace {
void cause_sigsegv() {
    cpputils::showBacktraceOnSigSegv();
    int* ptr = nullptr;
    int a = *ptr;
    (void)a;
}
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv) {
    EXPECT_DEATH(
        cause_sigsegv(),
        "cpputils::backtrace"
    );
}
