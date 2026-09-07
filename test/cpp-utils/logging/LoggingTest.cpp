#include "testutils/LoggingTest.h"
#include <regex>

/*
 * Contains test cases for the following logging interface:
 *   LOG(INFO, "My log message)"
 */

using namespace cpputils::logging;
using std::string;

void logAndExit(const string &message) {
    LOG(INFO, message);
    cpputils::logging::flush();
    exit(1);
}

void setLoggerAndLogAndExit(const string &message) {
    setLogger(spdlog::stderr_logger_mt("MyTestLog2"));
    LOG(INFO, message);
    cpputils::logging::flush();
    exit(1);
}

// The next two tests log in a child process instead of capturing stderr in
// this one. On Windows, spdlog's stderr sink looks up the Win32 handle behind
// the stderr file descriptor once, when the sink is created, and then writes
// to that handle with WriteFile(). Redirecting the stderr file descriptor
// afterwards - which is all gtest's stderr capture does - therefore doesn't
// reach the sink. A child process starts with its stderr already pointing at
// the pipe the death test reads, so the handle the sink caches is the one we
// are looking at.
TEST_F(LoggingTest, DefaultLoggerIsStderr) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_EXIT(
        logAndExit("My log message"),
        ::testing::ExitedWithCode(1),
        ::testing::HasSubstr("[Log] [info] My log message")
    );
}

TEST_F(LoggingTest, SetLogger_NewLoggerIsUsed) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_EXIT(
        setLoggerAndLogAndExit("My log message"),
        ::testing::ExitedWithCode(1),
        ::testing::HasSubstr("[MyTestLog2] [info] My log message")
    );
}

TEST_F(LoggingTest, SetNonStderrLogger_LogsToNewLogger) {
    setLogger(mockLogger.get());
    logger()->info("My log message");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(output, MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message.*")));
}

TEST_F(LoggingTest, SetNonStderrLogger_DoesNotLogToStderr) {
    setLogger(mockLogger.get());
    const string output = captureStderr([] {
        logger()->info("My log message");
        cpputils::logging::flush();
    });
    EXPECT_EQ("", output);
}

TEST_F(LoggingTest, InfoLog) {
    setLogger(mockLogger.get());
    LOG(INFO, "My log message");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message.*")));
}

TEST_F(LoggingTest, WarningLog) {
    setLogger(mockLogger.get());
    LOG(WARN, "My log message");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[warning\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[warning\\].*My log message.*")));
}

TEST_F(LoggingTest, DebugLog) {
    setLevel(DEBUG);
    setLogger(mockLogger.get());
    LOG(DEBUG, "My log message");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[debug\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[debug\\].*My log message.*")));
}

TEST_F(LoggingTest, ErrorLog) {
    setLogger(mockLogger.get());
    LOG(ERR, "My log message");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[error\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[error\\].*My log message.*")));
}

// fork() only forks the main thread. This test ensures that logging doesn't depend on threads that suddenly aren't
// there anymore after a fork().
TEST_F(LoggingTest, LoggingAlsoWorksAfterFork) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    setLogger(spdlog::stderr_logger_mt("StderrLogger"));
    EXPECT_EXIT(
        logAndExit("My log message"),
        ::testing::ExitedWithCode(1),
        "My log message"
    );
}

TEST_F(LoggingTest, MessageIsConstChar) {
    setLogger(mockLogger.get());
    LOG(INFO, "My log message");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message.*")));
}

TEST_F(LoggingTest, MessageIsString) {
    setLogger(mockLogger.get());
    const string msg = "My log message";
    LOG(INFO, msg);
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message.*")));
}

TEST_F(LoggingTest, FormatWithStringPlaceholder) {
    setLogger(mockLogger.get());
    const string str = "placeholder";
    LOG(INFO, "My log message: {}", str);
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message: placeholder.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message: placeholder.*")));
}

TEST_F(LoggingTest, FormatWithConstCharPlaceholder) {
    setLogger(mockLogger.get());
    LOG(INFO, "My log message: {}", "placeholder");
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message: placeholder.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message: placeholder.*")));
}

TEST_F(LoggingTest, FormatWithIntPlaceholder) {
    setLogger(mockLogger.get());
    LOG(INFO, "My log message: {}", 4);
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message: 4.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message: 4.*")));
}

TEST_F(LoggingTest, FormatWithMultiplePlaceholders) {
    setLogger(mockLogger.get());
    LOG(INFO, "My log message: {}, {}, {}", 4, "then", true);
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message: 4, then, true.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message: 4, then, true.*")));
}
