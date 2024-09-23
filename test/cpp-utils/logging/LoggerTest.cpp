#include "cpp-utils/logging/Logger.h"
#include "testutils/LoggingTest.h"
#include <gtest/gtest.h>
#include <spdlog/sinks/stdout_sinks.h>
#include <string>

/*
 * Contains test cases for the Logger class
 */

using namespace cpputils::logging;
using std::string;

class LoggerTest: public LoggingTest {};

TEST_F(LoggerTest, IsSingleton) {
    ASSERT_EQ(&logger(), &logger());
}

TEST_F(LoggerTest, SetLogger) {
    logger().setLogger(spdlog::stderr_logger_mt("MyTestLog1"));
    EXPECT_EQ("MyTestLog1", logger()->name());
}
