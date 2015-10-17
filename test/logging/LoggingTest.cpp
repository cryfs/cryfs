#include "testutils/LoggingTest.h"

/*
 * Contains test cases for the following logging interface:
 *   LOG(INFO) << "My log message"
 */

using namespace cpputils::logging;
using std::string;
using testing::MatchesRegex;

TEST_F(LoggingTest, DefaultLoggerIsStdout) {
    string output = captureStdout([]{
        LOG(INFO) << "My log message";
    });
    EXPECT_THAT(output, MatchesRegex(".*\\[Log\\].*\\[info\\].*My log message.*"));
}

TEST_F(LoggingTest, SetLogger_NewLoggerIsUsed) {
    setLogger(spdlog::stdout_logger_mt("MyTestLog2"));
    string output = captureStdout([]{
        LOG(INFO) << "My log message";
    });
    EXPECT_THAT(output, MatchesRegex(".*\\[MyTestLog2\\].*\\[info\\].*My log message.*"));
}

TEST_F(LoggingTest, SetNonStdoutLogger_LogsToNewLogger) {
    setLogger(mockLogger.get());
    logger()->info() << "My log message";
    EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
}

TEST_F(LoggingTest, SetNonStdoutLogger_DoesNotLogToStdout) {
    setLogger(mockLogger.get());
    string output = captureStdout([] {
        logger()->info() << "My log message";
    });
    EXPECT_EQ("", output);
}

TEST_F(LoggingTest, InfoLog) {
    setLogger(mockLogger.get());
    LOG(INFO) << "My log message";
    EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
}

TEST_F(LoggingTest, WarningLog) {
    setLogger(mockLogger.get());
    LOG(WARN) << "My log message";
    EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[warning\\].*My log message.*"));
}

TEST_F(LoggingTest, DebugLog) {
    setLevel(DEBUG);
    setLogger(mockLogger.get());
    LOG(DEBUG) << "My log message";
    EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[debug\\].*My log message.*"));
}

TEST_F(LoggingTest, ErrorLog) {
    setLogger(mockLogger.get());
    LOG(ERROR) << "My log message";
    EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[error\\].*My log message.*"));
}
