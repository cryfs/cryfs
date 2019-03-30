#include <gmock/gmock.h>
#include <csignal>
#include "cpp-utils/assert/backtrace.h"
#include "cpp-utils/process/subprocess.h"
#include <boost/filesystem.hpp>
#include "my-gtest-main.h"

using std::string;
using testing::HasSubstr;
namespace bf = boost::filesystem;

namespace {
	std::string call_process_exiting_with(const std::string& kind, const std::string& signal = "") {
#if defined(_MSC_VER)
		auto executable = get_executable().parent_path() / "cpp-utils-test_exit_signal.exe";
#else
		auto executable = get_executable().parent_path() / "cpp-utils-test_exit_signal";
#endif
		if (!bf::exists(executable)) {
			throw std::runtime_error(executable.string() + " not found.");
		}
		const std::string command = executable.string() + " \"" + kind + "\" \"" + signal + "\"  2>&1";
		auto result = cpputils::Subprocess::call(command);
		return result.output;
	}
}

#if !(defined(_MSC_VER) && defined(NDEBUG))

TEST(BacktraceTest, ContainsTopLevelLine) {
    string backtrace = cpputils::backtrace();
    EXPECT_THAT(backtrace, HasSubstr("BacktraceTest"));
    EXPECT_THAT(backtrace, HasSubstr("ContainsTopLevelLine"));
}
#endif

namespace {
	std::string call_process_exiting_with_nullptr_violation() {
		return call_process_exiting_with("nullptr");
	}
	std::string call_process_exiting_with_exception(const std::string& message) {
		return call_process_exiting_with("exception", message);
	}
}
#if defined(_MSC_VER)
#include <Windows.h>
namespace {
	std::string call_process_exiting_with_sigsegv() {
		return call_process_exiting_with("signal", std::to_string(EXCEPTION_ACCESS_VIOLATION));
	}
	std::string call_process_exiting_with_sigill() {
		return call_process_exiting_with("signal", std::to_string(EXCEPTION_ILLEGAL_INSTRUCTION));
	}
	std::string call_process_exiting_with_code(DWORD code) {
		return call_process_exiting_with("signal", std::to_string(code));
	}
}
#else
namespace {
	std::string call_process_exiting_with_sigsegv() {
		return call_process_exiting_with("signal", std::to_string(SIGSEGV));
	}
	std::string call_process_exiting_with_sigabrt() {
		return call_process_exiting_with("signal", std::to_string(SIGABRT));
	}
	std::string call_process_exiting_with_sigill() {
		return call_process_exiting_with("signal", std::to_string(SIGILL));
	}
}
#endif

TEST(BacktraceTest, DoesntCrashOnCaughtException) {
	// This is needed to make sure we don't use some kind of vectored exception handler on Windows
	// that ignores the call stack and always jumps on when an exception happens.
	cpputils::showBacktraceOnCrash();
	try {
		throw std::logic_error("exception");
	} catch (const std::logic_error& e) {
		// intentionally empty
	}
}

#if !(defined(_MSC_VER) && defined(NDEBUG))
TEST(BacktraceTest, ContainsBacktrace) {
    string backtrace = cpputils::backtrace();
#if defined(_MSC_VER)
    EXPECT_THAT(backtrace, HasSubstr("testing::Test::Run"));
#else
    EXPECT_THAT(backtrace, HasSubstr("BacktraceTest_ContainsBacktrace_Test::TestBody"));
#endif
}

TEST(BacktraceTest, ShowBacktraceOnNullptrAccess) {
	auto output = call_process_exiting_with_nullptr_violation();
    EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv) {
	auto output = call_process_exiting_with_sigsegv();
    EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
}

TEST(BacktraceTest, ShowBacktraceOnUnhandledException) {
	auto output = call_process_exiting_with_exception("my_exception_message");
    EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
}

TEST(BacktraceTest, ShowBacktraceOnSigIll) {
	auto output = call_process_exiting_with_sigill();
    EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
}
#else
TEST(BacktraceTest, ContainsBacktrace) {
	string backtrace = cpputils::backtrace();
	EXPECT_THAT(backtrace, HasSubstr("#1"));
}
TEST(BacktraceTest, ShowBacktraceOnNullptrAccess) {
	auto output = call_process_exiting_with_nullptr_violation();
	EXPECT_THAT(output, HasSubstr("#1"));
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv) {
	auto output = call_process_exiting_with_sigsegv();
	EXPECT_THAT(output, HasSubstr("#1"));
}

TEST(BacktraceTest, ShowBacktraceOnUnhandledException) {
	auto output = call_process_exiting_with_exception("my_exception_message");
	EXPECT_THAT(output, HasSubstr("#1"));
}

TEST(BacktraceTest, ShowBacktraceOnSigIll) {
	auto output = call_process_exiting_with_sigill();
	EXPECT_THAT(output, HasSubstr("#1"));
}
#endif

#if !defined(_MSC_VER)
TEST(BacktraceTest, ShowBacktraceOnSigAbrt) {
	auto output = call_process_exiting_with_sigabrt();
	EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
}

TEST(BacktraceTest, ShowBacktraceOnSigAbrt_ShowsCorrectSignalName) {
	auto output = call_process_exiting_with_sigabrt();
	EXPECT_THAT(output, HasSubstr("SIGABRT"));
}
#endif

constexpr const char* sigsegv_message = "SIGSEGV";
constexpr const char* sigill_message = "SIGILL";

TEST(BacktraceTest, ShowBacktraceOnSigSegv_ShowsCorrectSignalName) {
	auto output = call_process_exiting_with_sigsegv();
	EXPECT_THAT(output, HasSubstr(sigsegv_message));
}

TEST(BacktraceTest, ShowBacktraceOnSigIll_ShowsCorrectSignalName) {
	auto output = call_process_exiting_with_sigill();
	EXPECT_THAT(output, HasSubstr(sigill_message));
}

#if !defined(_MSC_VER)
TEST(BacktraceTest, ShowBacktraceOnUnhandledException_ShowsCorrectExceptionMessage) {
	auto output = call_process_exiting_with_exception("my_exception_message");
	EXPECT_THAT(output, HasSubstr("my_exception_message"));
}
#endif

