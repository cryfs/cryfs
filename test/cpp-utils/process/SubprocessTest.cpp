#include <cpp-utils/process/subprocess.h>
#include <gtest/gtest.h>
#include <boost/filesystem.hpp>

#include <cpp-utils/lock/ConditionBarrier.h>
#include "my-gtest-main.h"

using cpputils::Subprocess;
using cpputils::SubprocessError;
using std::string;
namespace bf = boost::filesystem;

namespace {
std::string exit_with_message_and_status(const char* message, int status) {
#if defined(_MSC_VER)
    auto executable = get_executable().parent_path() / "cpp-utils-test_exit_status.exe";
#else
    auto executable = get_executable().parent_path() / "cpp-utils-test_exit_status";
#endif
    if (!bf::exists(executable)) {
        throw std::runtime_error(executable.string() + " not found.");
    }
	return executable.string() + " \"" + message + "\" " + std::to_string(status);
}
}

TEST(SubprocessTest, CheckCall_success_output) {
    EXPECT_EQ("hello", Subprocess::check_call(exit_with_message_and_status("hello", 0)).output);
}

TEST(SubprocessTest, CheckCall_successwithemptyoutput_output) {
    EXPECT_EQ("", Subprocess::check_call(exit_with_message_and_status("", 0)).output);
}

TEST(SubprocessTest, CheckCall_success_exitcode) {
    EXPECT_EQ(0, Subprocess::check_call(exit_with_message_and_status("hello", 0)).exitcode);
}

TEST(SubprocessTest, CheckCall_successwithemptyoutput_exitcode) {
    EXPECT_EQ(0, Subprocess::check_call(exit_with_message_and_status("", 0)).exitcode);
}

TEST(SubprocessTest, CheckCall_error) {
    EXPECT_THROW(
      Subprocess::check_call(exit_with_message_and_status("", 1)),
      SubprocessError
    );
}

TEST(SubprocessTest, CheckCall_error5) {
    EXPECT_THROW(
      Subprocess::check_call(exit_with_message_and_status("", 5)),
      SubprocessError
    );
}

TEST(SubprocessTest, CheckCall_errorwithoutput) {
    EXPECT_THROW(
      Subprocess::check_call(exit_with_message_and_status("hello", 1)),
      SubprocessError
    );
}

TEST(SubprocessTest, CheckCall_error5withoutput) {
    EXPECT_THROW(
      Subprocess::check_call(exit_with_message_and_status("hello", 5)),
      SubprocessError
    );
}

TEST(SubprocessTest, Call_success_exitcode) {
    EXPECT_EQ(0, Subprocess::call(exit_with_message_and_status("hello", 0)).exitcode);
}

TEST(SubprocessTest, Call_success_output) {
    EXPECT_EQ("hello", Subprocess::call(exit_with_message_and_status("hello", 0)).output);
}

TEST(SubprocessTest, Call_error_exitcode) {
    EXPECT_EQ(1, Subprocess::call(exit_with_message_and_status("", 1)).exitcode);
}

TEST(SubprocessTest, Call_error_output) {
    EXPECT_EQ("", Subprocess::call(exit_with_message_and_status("", 1)).output);
}

TEST(SubprocessTest, Call_error5_exitcode) {
    EXPECT_EQ(5, Subprocess::call(exit_with_message_and_status("", 5)).exitcode);
}

TEST(SubprocessTest, Call_error5_output) {
    EXPECT_EQ("", Subprocess::call(exit_with_message_and_status("", 1)).output);
}

TEST(SubprocessTest, Call_errorwithoutput_output) {
    EXPECT_EQ("hello", Subprocess::call(exit_with_message_and_status("hello", 1)).output);
}

TEST(SubprocessTest, Call_errorwithoutput_exitcode) {
    EXPECT_EQ(1, Subprocess::call(exit_with_message_and_status("hello", 1)).exitcode);
}

TEST(SubprocessTest, Call_error5withoutput_output) {
    EXPECT_EQ("hello", Subprocess::call(exit_with_message_and_status("hello", 5)).output);
}

TEST(SubprocessTest, Call_error5withoutput_exitcode) {
    EXPECT_EQ(5, Subprocess::call(exit_with_message_and_status("hello", 5)).exitcode);
}

// TODO Move this test to a test suite for ThreadSystem/LoopThread
#include <cpp-utils/thread/LoopThread.h>
TEST(SubprocessTest, CallFromThreadSystemThread) {
    cpputils::ConditionBarrier barrier;

    cpputils::LoopThread thread(
        [&barrier] () {
            auto result = Subprocess::check_call(exit_with_message_and_status("hello", 0));
            EXPECT_EQ(0, result.exitcode);
            EXPECT_EQ("hello", result.output);

            barrier.release();

            return false; // don't run loop again
        },
        "child_thread"
    );
    thread.start();
    barrier.wait();
    thread.stop(); // just to make sure it's stopped before the test exits. Returning false above should already stop it, but we don't know when exactly. thread.stop() will block until it's actually stopped.
}
