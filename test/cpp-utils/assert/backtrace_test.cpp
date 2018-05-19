#include <gmock/gmock.h>
#include <csignal>
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
void nullptr_access() {
    cpputils::showBacktraceOnCrash();
    int* ptr = nullptr;
    *ptr = 5;
}
void raise_signal(int signal) {
    cpputils::showBacktraceOnCrash();
    ::raise(signal);
}
}

TEST(BacktraceTest, ShowBacktraceOnNullptrAccess) {
    EXPECT_DEATH(
        nullptr_access(),
        "cpputils::backtrace"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv) {
    EXPECT_DEATH(
            raise_signal(SIGSEGV),
            "cpputils::backtrace"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigAbrt) {
    EXPECT_DEATH(
            raise_signal(SIGABRT),
            "cpputils::backtrace"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigIll) {
    EXPECT_DEATH(
            raise_signal(SIGILL),
            "cpputils::backtrace"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv_ShowsCorrectSignalName) {
    EXPECT_DEATH(
            raise_signal(SIGSEGV),
            "SIGSEGV"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigAbrt_ShowsCorrectSignalName) {
    EXPECT_DEATH(
            raise_signal(SIGABRT),
            "SIGABRT"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigIll_ShowsCorrectSignalName) {
    EXPECT_DEATH(
            raise_signal(SIGILL),
            "SIGILL"
    );
}
