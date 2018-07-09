#include <cpp-utils/process/subprocess.h>
#include <gtest/gtest.h>

using cpputils::Subprocess;
using cpputils::SubprocessError;

namespace {
std::string exit_with_message_and_status(const char* message, int status) {
#if defined(_MSC_VER)
    constexpr const char* executable = "cpp-utils-test_exit_status.exe";
#else
    constexpr const char* executable = "./test/cpp-utils/cpp-utils-test_exit_status";
#endif
	return std::string(executable) + " \"" + message + "\" " + std::to_string(status);
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
