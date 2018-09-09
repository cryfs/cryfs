#include <cpp-utils/io/DontEchoStdinToStdoutRAII.h>
#include <gtest/gtest.h>

using cpputils::DontEchoStdinToStdoutRAII;

TEST(DontEchoStdinToStdoutRAIITest, DoesntCrash) {
    DontEchoStdinToStdoutRAII a;
}
