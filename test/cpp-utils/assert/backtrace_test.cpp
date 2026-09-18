#include <gmock/gmock.h>
#include <csignal>
#include "cpp-utils/assert/backtrace.h"
#include "cpp-utils/process/subprocess.h"
#include <boost/filesystem.hpp>
#include "my-gtest-main.h"

#if !defined(_MSC_VER)
#include <sys/wait.h>
#endif

using std::string;
using testing::HasSubstr;
using testing::Not;
namespace bf = boost::filesystem;

namespace
{
	cpputils::SubprocessResult run_process_exiting_with(const std::string &kind, const std::string &signal)
	{
#if defined(_MSC_VER)
		auto executable = bf::canonical(get_executable().parent_path()) / "cpp-utils-test_exit_signal.exe";
#else
		auto executable = bf::canonical(get_executable().parent_path()) / "cpp-utils-test_exit_signal";
#endif
		if (!bf::exists(executable))
		{
			throw std::runtime_error(executable.string() + " not found.");
		}
		return cpputils::Subprocess::call(executable, {kind, signal}, "");
	}

	std::string call_process_exiting_with(const std::string &kind, const std::string &signal = "")
	{
		return run_process_exiting_with(kind, signal).output_stderr;
	}
}

#if !(defined(_MSC_VER) && defined(NDEBUG))

TEST(BacktraceTest, ContainsTopLevelLine)
{
	const string backtrace = cpputils::backtrace();
	EXPECT_THAT(backtrace, HasSubstr("BacktraceTest"));
	EXPECT_THAT(backtrace, HasSubstr("ContainsTopLevelLine"));
}
#endif

namespace
{
#if !(defined(__clang__)) || (defined(_MSC_VER) && defined(NDEBUG))
	std::string call_process_exiting_with_nullptr_violation()
	{
		return call_process_exiting_with("nullptr");
	}
#endif
	std::string call_process_exiting_with_exception(const std::string &message)
	{
		return call_process_exiting_with("exception", message);
	}
}

#if defined(_MSC_VER)
#include <Windows.h>
namespace
{
	std::string call_process_exiting_with_sigsegv()
	{
		return call_process_exiting_with("signal", std::to_string(EXCEPTION_ACCESS_VIOLATION));
	}
	std::string call_process_exiting_with_sigill()
	{
		return call_process_exiting_with("signal", std::to_string(EXCEPTION_ILLEGAL_INSTRUCTION));
	}
	std::string call_process_exiting_with_code(DWORD code)
	{
		return call_process_exiting_with("signal", std::to_string(code));
	}
}
#else
namespace
{
	std::string call_process_exiting_with_sigsegv()
	{
		return call_process_exiting_with("signal", std::to_string(SIGSEGV));
	}
	std::string call_process_exiting_with_sigabrt()
	{
		return call_process_exiting_with("signal", std::to_string(SIGABRT));
	}
	std::string call_process_exiting_with_sigill()
	{
		return call_process_exiting_with("signal", std::to_string(SIGILL));
	}
}
#endif

TEST(BacktraceTest, DoesntCrashOnCaughtException)
{
	// This is needed to make sure we don't use some kind of vectored exception handler on Windows
	// that ignores the call stack and always jumps on when an exception happens.
	cpputils::showBacktraceOnCrash();
	try
	{
		throw std::logic_error("exception");
	}
	catch (const std::logic_error &e) // NOLINT(bugprone-empty-catch)
	{
		// intentionally empty
	}
}

#if !(defined(_MSC_VER) && defined(NDEBUG))
TEST(BacktraceTest, ContainsBacktrace)
{
	const string backtrace = cpputils::backtrace();
#if defined(_MSC_VER)
	EXPECT_THAT(backtrace, HasSubstr("testing::Test::Run"));
#else
	EXPECT_THAT(backtrace, HasSubstr("BacktraceTest_ContainsBacktrace_Test::TestBody"));
#endif
}

#if !(defined(__clang__))
// TODO Can we also make this work on clang?
TEST(BacktraceTest, ShowBacktraceOnNullptrAccess)
{
	auto output = call_process_exiting_with_nullptr_violation();
#if defined(_MSC_VER)
	EXPECT_THAT(output, HasSubstr("handle_exit_signal"));
#else
	EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
#endif
}
#endif

TEST(BacktraceTest, ShowBacktraceOnSigSegv)
{
	auto output = call_process_exiting_with_sigsegv();
#if defined(_MSC_VER)
	EXPECT_THAT(output, HasSubstr("handle_exit_signal"));
#else
	EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
#endif
}

TEST(BacktraceTest, ShowBacktraceOnUnhandledException)
{
	auto output = call_process_exiting_with_exception("my_exception_message");
#if defined(_MSC_VER)
	EXPECT_THAT(output, HasSubstr("handle_exit_signal"));
#else
	EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
#endif
}

TEST(BacktraceTest, ShowBacktraceOnSigIll)
{
	auto output = call_process_exiting_with_sigill();
#if defined(_MSC_VER)
	EXPECT_THAT(output, HasSubstr("handle_exit_signal"));
#else
	EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
#endif
}
#else
TEST(BacktraceTest, ContainsBacktrace)
{
	string backtrace = cpputils::backtrace();
	EXPECT_THAT(backtrace, HasSubstr("#0"));
}
TEST(BacktraceTest, ShowBacktraceOnNullptrAccess)
{
	auto output = call_process_exiting_with_nullptr_violation();
	EXPECT_THAT(output, HasSubstr("#1"));
}

TEST(BacktraceTest, ShowBacktraceOnSigSegv)
{
	auto output = call_process_exiting_with_sigsegv();
	EXPECT_THAT(output, HasSubstr("#1"));
}

TEST(BacktraceTest, ShowBacktraceOnUnhandledException)
{
	auto output = call_process_exiting_with_exception("my_exception_message");
	EXPECT_THAT(output, HasSubstr("#1"));
}

TEST(BacktraceTest, ShowBacktraceOnSigIll)
{
	auto output = call_process_exiting_with_sigill();
	EXPECT_THAT(output, HasSubstr("#1"));
}
#endif

#if !defined(_MSC_VER)
TEST(BacktraceTest, ShowBacktraceOnSigAbrt)
{
	auto output = call_process_exiting_with_sigabrt();
	EXPECT_THAT(output, HasSubstr("cpputils::backtrace"));
}

TEST(BacktraceTest, ShowBacktraceOnSigAbrt_ShowsCorrectSignalName)
{
	auto output = call_process_exiting_with_sigabrt();
	EXPECT_THAT(output, HasSubstr("SIGABRT"));
}
#endif

#if !defined(_MSC_VER)
constexpr const char *sigsegv_message = "SIGSEGV";
constexpr const char *sigill_message = "SIGILL";
#else
constexpr const char *sigsegv_message = "EXCEPTION_ACCESS_VIOLATION";
constexpr const char *sigill_message = "EXCEPTION_ILLEGAL_INSTRUCTION";
#endif

TEST(BacktraceTest, ShowBacktraceOnSigSegv_ShowsCorrectSignalName)
{
	auto output = call_process_exiting_with_sigsegv();
	EXPECT_THAT(output, HasSubstr(sigsegv_message));
}

TEST(BacktraceTest, ShowBacktraceOnSigIll_ShowsCorrectSignalName)
{
	auto output = call_process_exiting_with_sigill();
	EXPECT_THAT(output, HasSubstr(sigill_message));
}

#if !defined(_MSC_VER)
TEST(BacktraceTest, ShowBacktraceOnUnhandledException_ShowsCorrectExceptionMessage)
{
	auto output = call_process_exiting_with_exception("my_exception_message");
	EXPECT_THAT(output, HasSubstr("my_exception_message"));
}
#endif

#if defined(_MSC_VER)
TEST(BacktraceTest, UnknownCode_ShowsCorrectSignalName)
{
	auto output = call_process_exiting_with_code(0x1234567);
	EXPECT_THAT(output, HasSubstr("UNKNOWN_CODE(0x1234567)"));
}
#endif

#if !defined(_MSC_VER)
// The following tests are for showBacktraceOnCrashSignals(), which is about the POSIX crash
// signals. On Windows, that function just installs the top level exception filter, i.e. exactly
// what the showBacktraceOnCrash() tests above already cover.
namespace
{
	void expect_shows_backtrace_and_dies_from_signal(int signal, const std::string &signal_name)
	{
		const auto result = run_process_exiting_with("crash_signal", std::to_string(signal));
		EXPECT_THAT(result.output_stderr, HasSubstr(signal_name));
		// Check for the frame of the function that crashed, not for one of the backtrace machinery
		// itself (e.g. "cpputils::backtrace"), which would be in there no matter where we crashed.
		EXPECT_THAT(result.output_stderr, HasSubstr("handle_exit_signal"));
		// The signal has to kill us, otherwise we'd throw away the core dump and hide from our
		// caller that we crashed at all.
		ASSERT_TRUE(WIFSIGNALED(result.native_exit_code)) << "Expected the process to be killed by a signal but it wasn't. Its stderr was:\n"
														  << result.output_stderr;
		EXPECT_EQ(signal, WTERMSIG(result.native_exit_code));
	}
}

TEST(BacktraceTest, ShowBacktraceOnCrashSignals_SigSegv)
{
	expect_shows_backtrace_and_dies_from_signal(SIGSEGV, "SIGSEGV");
}

TEST(BacktraceTest, ShowBacktraceOnCrashSignals_SigIll)
{
	expect_shows_backtrace_and_dies_from_signal(SIGILL, "SIGILL");
}

TEST(BacktraceTest, ShowBacktraceOnCrashSignals_SigBus)
{
	expect_shows_backtrace_and_dies_from_signal(SIGBUS, "SIGBUS");
}

TEST(BacktraceTest, ShowBacktraceOnCrashSignals_SigFpe)
{
	expect_shows_backtrace_and_dies_from_signal(SIGFPE, "SIGFPE");
}

// ASSERT() and the gtest death tests abort() on purpose, so we have to leave SIGABRT alone.
TEST(BacktraceTest, ShowBacktraceOnCrashSignals_DoesntHandleSigAbrt)
{
	const auto result = run_process_exiting_with("crash_signal", std::to_string(SIGABRT));
	EXPECT_THAT(result.output_stderr, Not(HasSubstr("SIGABRT")));
	ASSERT_TRUE(WIFSIGNALED(result.native_exit_code)) << "Expected the process to be killed by a signal but it wasn't. Its stderr was:\n"
													  << result.output_stderr;
	EXPECT_EQ(SIGABRT, WTERMSIG(result.native_exit_code));
}
#endif
