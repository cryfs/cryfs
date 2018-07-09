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
}

#if defined(_MSC_VER)
#include <Windows.h>
namespace {
	void raise_sigsegv() {
		cpputils::showBacktraceOnCrash();
		::RaiseException(EXCEPTION_ACCESS_VIOLATION, EXCEPTION_NONCONTINUABLE, 0, NULL);
	}
	void raise_sigill() {
		cpputils::showBacktraceOnCrash();
		::RaiseException(EXCEPTION_ILLEGAL_INSTRUCTION, EXCEPTION_NONCONTINUABLE, 0, NULL);
	}
	void raise_code(DWORD exception_code) {
		cpputils::showBacktraceOnCrash();
		::RaiseException(exception_code, EXCEPTION_NONCONTINUABLE, 0, NULL);
	}
}
#else
namespace {
	void raise_sigsegv() {
		cpputils::showBacktraceOnCrash();
		::raise(SIGSEGV);
	}
	void raise_sigabrt() {
		cpputils::showBacktraceOnCrash();
		::raise(SIGABRT);
	}
	void raise_sigill() {
		cpputils::showBacktraceOnCrash();
		::raise(SIGILL);
	}
}
#endif

TEST(BacktraceTest, ShowBacktraceOnNullptrAccess) {
    EXPECT_DEATH(
		nullptr_access(),
        "ShowBacktraceOnNullptrAccess_Test::TestBody"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv) {
    EXPECT_DEATH(
		raise_sigsegv(),
        "ShowBacktraceOnSigSegv_Test::TestBody"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigIll) {
	EXPECT_DEATH(
		raise_sigill(),
		"ShowBacktraceOnSigIll_Test::TestBody"
	);
}

#if !defined(_MSC_VER)
TEST(BacktraceTest, ShowBacktraceOnSigAbrt) {
    EXPECT_DEATH(
        raise_sigabrt(),
        "ShowBacktraceOnSigAbrt_Test::TestBody"
    );
}

TEST(BacktraceTest, ShowBacktraceOnSigAbrt_ShowsCorrectSignalName) {
	EXPECT_DEATH(
		raise_sigabrt(),
		"SIGABRT"
	);
}
#endif

#if !defined(_MSC_VER)
constexpr const char* sigsegv_message = "SIGSEGV";
constexpr const char* sigill_message = "SIGILL";
#else
constexpr const char* sigsegv_message = "EXCEPTION_ACCESS_VIOLATION";
constexpr const char* sigill_message = "EXCEPTION_ILLEGAL_INSTRUCTION";
#endif

TEST(BacktraceTest, ShowBacktraceOnSigSegv_ShowsCorrectSignalName) {
	EXPECT_DEATH(
		raise_sigsegv(),
		sigsegv_message
	);
}

TEST(BacktraceTest, ShowBacktraceOnSigIll_ShowsCorrectSignalName) {
    EXPECT_DEATH(
		raise_sigill(),
		sigill_message
    );
}

#if defined(_MSC_VER)
TEST(BacktraceTest, UnknownCode_ShowsCorrectSignalName) {
	EXPECT_DEATH(
		raise_code(0x12345678),
		"UNKNOWN_CODE\\(0x12345678\\)"
	);
}
#endif
