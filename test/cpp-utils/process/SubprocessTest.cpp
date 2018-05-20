#include <cpp-utils/process/subprocess.h>
#include <gtest/gtest.h>

using cpputils::Subprocess;
using cpputils::SubprocessError;

TEST(SubprocessTest, CheckCall_success_output) {
    EXPECT_EQ("hello\n", Subprocess::check_call("echo hello").output);
}

TEST(SubprocessTest, CheckCall_successwithemptyoutput_output) {
    EXPECT_EQ("", Subprocess::check_call("exit 0").output);
}

TEST(SubprocessTest, CheckCall_success_exitcode) {
    EXPECT_EQ(0, Subprocess::check_call("echo hello").exitcode);
}

TEST(SubprocessTest, CheckCall_successwithemptyoutput_exitcode) {
    EXPECT_EQ(0, Subprocess::check_call("exit 0").exitcode);
}

TEST(SubprocessTest, CheckCall_error) {
    EXPECT_THROW(
      Subprocess::check_call("exit 1"),
      SubprocessError
    );
}

TEST(SubprocessTest, CheckCall_error5) {
    EXPECT_THROW(
      Subprocess::check_call("exit 5"),
      SubprocessError
    );
}

TEST(SubprocessTest, CheckCall_errorwithoutput) {
    EXPECT_THROW(
      Subprocess::check_call("echo hello; exit 1"),
      SubprocessError
    );
}

TEST(SubprocessTest, CheckCall_error5withoutput) {
    EXPECT_THROW(
      Subprocess::check_call("echo hello; exit 5"),
      SubprocessError
    );
}

TEST(SubprocessTest, Call_success_exitcode) {
    EXPECT_EQ(0, Subprocess::call("echo hello").exitcode);
}

TEST(SubprocessTest, Call_success_output) {
    EXPECT_EQ("hello\n", Subprocess::call("echo hello").output);
}

TEST(SubprocessTest, Call_error_exitcode) {
    EXPECT_EQ(1, Subprocess::call("exit 1").exitcode);
}

TEST(SubprocessTest, Call_error_output) {
    EXPECT_EQ("", Subprocess::call("exit 1").output);
}

TEST(SubprocessTest, Call_error5_exitcode) {
    EXPECT_EQ(5, Subprocess::call("exit 5").exitcode);
}

TEST(SubprocessTest, Call_error5_output) {
    EXPECT_EQ("", Subprocess::call("exit 1").output);
}

TEST(SubprocessTest, Call_errorwithoutput_output) {
    EXPECT_EQ("hello\n", Subprocess::call("echo hello; exit 1").output);
}

TEST(SubprocessTest, Call_errorwithoutput_exitcode) {
    EXPECT_EQ(1, Subprocess::call("echo hello; exit 1").exitcode);
}

TEST(SubprocessTest, Call_error5withoutput_output) {
    EXPECT_EQ("hello\n", Subprocess::call("echo hello; exit 5").output);
}

TEST(SubprocessTest, Call_error5withoutput_exitcode) {
    EXPECT_EQ(5, Subprocess::call("echo hello; exit 5").exitcode);
}
