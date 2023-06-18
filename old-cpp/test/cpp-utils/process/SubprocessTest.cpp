#include <cpp-utils/process/subprocess.h>
#include <gtest/gtest.h>
#include <boost/filesystem.hpp>

#include <cpp-utils/lock/ConditionBarrier.h>
#include "my-gtest-main.h"

using cpputils::Subprocess;
using cpputils::SubprocessError;
using std::string;
namespace bf = boost::filesystem;

// TODO Test passing input to stdin of processes
// TODO Test stderr

#if defined(_MSC_VER)
constexpr const char* NEWLINE = "\r\n";
#else
constexpr const char* NEWLINE = "\n";
#endif

namespace
{
    bf::path exit_with_message_and_status()
    {
#if defined(_MSC_VER)
        auto executable = bf::canonical(get_executable().parent_path()) / "cpp-utils-test_exit_status.exe";
#else
        auto executable = bf::canonical(get_executable().parent_path()) / "cpp-utils-test_exit_status";
#endif
        if (!bf::exists(executable))
        {
            throw std::runtime_error(executable.string() + " not found.");
        }
        return executable;
    }
}

TEST(SubprocessTest, CheckCall_success_output)
{
    EXPECT_EQ(std::string("hello") + NEWLINE, Subprocess::check_call(exit_with_message_and_status(), {"0", "hello"}, "").output_stdout);
}

TEST(SubprocessTest, CheckCall_successwithemptyoutput_output)
{
    EXPECT_EQ("", Subprocess::check_call(exit_with_message_and_status(), {"0"}, "").output_stdout);
}

TEST(SubprocessTest, CheckCall_success_exitcode)
{
    EXPECT_EQ(0, Subprocess::check_call(exit_with_message_and_status(), {"0", "hello"}, "").exitcode);
}

TEST(SubprocessTest, CheckCall_successwithemptyoutput_exitcode)
{
    EXPECT_EQ(0, Subprocess::check_call(exit_with_message_and_status(), {"0"}, "").exitcode);
}

TEST(SubprocessTest, CheckCall_error)
{
    EXPECT_THROW(
        Subprocess::check_call(exit_with_message_and_status(), {"1"}, ""),
        SubprocessError);
}

TEST(SubprocessTest, CheckCall_error5)
{
    EXPECT_THROW(
        Subprocess::check_call(exit_with_message_and_status(), {"5"}, ""),
        SubprocessError);
}

TEST(SubprocessTest, CheckCall_errorwithoutput)
{
    EXPECT_THROW(
        Subprocess::check_call(exit_with_message_and_status(), {"1", "hello"}, ""),
        SubprocessError);
}

TEST(SubprocessTest, CheckCall_error5withoutput)
{
    EXPECT_THROW(
        Subprocess::check_call(exit_with_message_and_status(), {"5", "hello"}, ""),
        SubprocessError);
}

TEST(SubprocessTest, Call_success_exitcode)
{
    EXPECT_EQ(0, Subprocess::call(exit_with_message_and_status(), {"0", "hello"}, "").exitcode);
}

TEST(SubprocessTest, Call_success_output)
{
    EXPECT_EQ(std::string("hello") + NEWLINE, Subprocess::call(exit_with_message_and_status(), {"0", "hello"}, "").output_stdout);
}

TEST(SubprocessTest, Call_error_exitcode)
{
    EXPECT_EQ(1, Subprocess::call(exit_with_message_and_status(), {"1"}, "").exitcode);
}

TEST(SubprocessTest, Call_error_output)
{
    EXPECT_EQ("", Subprocess::call(exit_with_message_and_status(), {"1"}, "").output_stdout);
}

TEST(SubprocessTest, Call_error5_exitcode)
{
    EXPECT_EQ(5, Subprocess::call(exit_with_message_and_status(), {"5"}, "").exitcode);
}

TEST(SubprocessTest, Call_error5_output)
{
    EXPECT_EQ("", Subprocess::call(exit_with_message_and_status(), {"1"}, "").output_stdout);
}

TEST(SubprocessTest, Call_errorwithoutput_output)
{
    EXPECT_EQ(std::string("hello") + NEWLINE, Subprocess::call(exit_with_message_and_status(), {"1", "hello"}, "").output_stdout);
}

TEST(SubprocessTest, Call_errorwithoutput_exitcode)
{
    EXPECT_EQ(1, Subprocess::call(exit_with_message_and_status(), {"1", "hello"}, "").exitcode);
}

TEST(SubprocessTest, Call_error5withoutput_output)
{
    EXPECT_EQ(std::string("hello") + NEWLINE, Subprocess::call(exit_with_message_and_status(), {"5", "hello"}, "").output_stdout);
}

TEST(SubprocessTest, Call_error5withoutput_exitcode)
{
    EXPECT_EQ(5, Subprocess::call(exit_with_message_and_status(), {"5", "hello"}, "").exitcode);
}

// TODO Move this test to a test suite for ThreadSystem/LoopThread
#include <cpp-utils/thread/LoopThread.h>
TEST(SubprocessTest, CallFromThreadSystemThread)
{
    cpputils::ConditionBarrier barrier;

    cpputils::LoopThread thread(
        [&barrier]()
        {
            auto result = Subprocess::check_call(exit_with_message_and_status(), {"0", "hello"}, "");
            EXPECT_EQ(0, result.exitcode);
            EXPECT_EQ(std::string("hello") + NEWLINE, result.output_stdout);

            barrier.release();

            return false; // don't run loop again
        },
        "child_thread");
    thread.start();
    barrier.wait();
    thread.stop(); // just to make sure it's stopped before the test exits. Returning false above should already stop it, but we don't know when exactly. thread.stop() will block until it's actually stopped.
}

TEST(SubprocessTest, Call_argumentwithspaces)
{
    // Test that arguments can have spaces and are still treated as one argument
    EXPECT_EQ(std::string("hello world") + NEWLINE, Subprocess::check_call(exit_with_message_and_status(), {"0", "hello world"}, "").output_stdout);
    EXPECT_EQ(std::string("hello") + NEWLINE + "world" + NEWLINE, Subprocess::check_call(exit_with_message_and_status(), {"0", "hello", "world"}, "").output_stdout);
}

#if !defined(_MSC_VER)
TEST(SubprocessTest, Call_withcommandfrompath)
{
    // Test that we can call a system command without specifying the full path
    EXPECT_EQ("hello\n", Subprocess::check_call("echo", {"hello"}, "").output_stdout);
}
#endif
