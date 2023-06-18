#include "testutils/LoggingTest.h"
#include <regex>

/*
 * Contains test cases for the following logging interface:
 *   LOG(INFO, "My log message)"
 */

using namespace cpputils::logging;
using std::string;

// Disable the next tests for MSVC debug builds since writing to stderr doesn't seem to work well there
#if !defined(_MSC_VER) || NDEBUG

TEST_F(LoggingTest, DefaultLoggerIsStderr) {
    string output = captureStderr([]{
        LOG(INFO, "My log message");
        cpputils::logging::flush();
    });
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
    //EXPECT_THAT(output, MatchesRegex(".*\\[Log\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(output, std::regex(".*\\[Log\\].*\\[info\\].*My log message.*")));
}

TEST_F(LoggingTest, SetLogger_NewLoggerIsUsed) {
    setLogger(spdlog::stderr_logger_mt("MyTestLog2"));
    string output = captureStderr([]{
        LOG(INFO, "My log message");
        cpputils::logging::flush();
    });
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(output, MatchesRegex(".*\\[MyTestLog2\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(output, std::regex(".*\\[MyTestLog2\\].*\\[info\\].*My log message.*")));
}

#endif

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
    string output = captureStderr([] {
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

void logAndExit(const string &message) {
    LOG(INFO, message);
    cpputils::logging::flush();
    exit(1);
}

// fork() only forks the main thread. This test ensures that logging doesn't depend on threads that suddenly aren't
// there anymore after a fork().
TEST_F(LoggingTest, LoggingAlsoWorksAfterFork) {
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
    string msg = "My log message";
    LOG(INFO, msg);
    cpputils::logging::flush();
	// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
	//EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
	EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message.*")));
}

TEST_F(LoggingTest, FormatWithStringPlaceholder) {
    setLogger(mockLogger.get());
    string str = "placeholder";
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
