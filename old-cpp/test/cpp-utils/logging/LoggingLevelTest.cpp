#include "testutils/LoggingTest.h"
#include <regex>

using namespace cpputils::logging;
using std::string;

class LoggingLevelTest: public LoggingTest {
public:
    void EXPECT_DEBUG_LOG_ENABLED() {
        LOG(DEBUG, "My log message");
		// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
        //EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[debug\\].*My log message.*"));
		EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[debug\\].*My log message.*")));
    }

    void EXPECT_DEBUG_LOG_DISABLED() {
        LOG(DEBUG, "My log message");
        EXPECT_EQ("", mockLogger.capturedLog());
    }

    void EXPECT_INFO_LOG_ENABLED() {
        LOG(INFO, "My log message");
		// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
        //EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[info\\].*My log message.*"));
		EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[info\\].*My log message.*")));
    }

    void EXPECT_INFO_LOG_DISABLED() {
        LOG(INFO, "My log message");
        EXPECT_EQ("", mockLogger.capturedLog());
    }

    void EXPECT_WARNING_LOG_ENABLED() {
        LOG(WARN, "My log message");
		EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[warning\\].*My log message.*")));
		// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
        //EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[warning\\].*My log message.*"));
    }

    void EXPECT_WARNING_LOG_DISABLED() {
        LOG(WARN, "My log message");
        EXPECT_EQ("", mockLogger.capturedLog());
    }

    void EXPECT_ERROR_LOG_ENABLED() {
        LOG(ERR, "My log message");
		// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
        //EXPECT_THAT(mockLogger.capturedLog(), MatchesRegex(".*\\[MockLogger\\].*\\[error\\].*My log message.*"));
		EXPECT_TRUE(std::regex_search(mockLogger.capturedLog(), std::regex(".*\\[MockLogger\\].*\\[error\\].*My log message.*")));
    }

    void EXPECT_ERROR_LOG_DISABLED() {
        LOG(ERR, "My log message");
        EXPECT_EQ("", mockLogger.capturedLog());
    }
};

TEST_F(LoggingLevelTest, DefaultLevelIsInfo) {
    setLogger(mockLogger.get());
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_ENABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, DEBUG_SetBeforeSettingLogger) {
    setLevel(DEBUG);
    setLogger(mockLogger.get());
    EXPECT_DEBUG_LOG_ENABLED();
    EXPECT_INFO_LOG_ENABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, DEBUG_SetAfterSettingLogger) {
    setLogger(mockLogger.get());
    setLevel(DEBUG);
    EXPECT_DEBUG_LOG_ENABLED();
    EXPECT_INFO_LOG_ENABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, INFO_SetBeforeSettingLogger) {
    setLevel(INFO);
    setLogger(mockLogger.get());
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_ENABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, INFO_SetAfterSettingLogger) {
    setLogger(mockLogger.get());
    setLevel(INFO);
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_ENABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, WARNING_SetBeforeSettingLogger) {
    setLevel(WARN);
    setLogger(mockLogger.get());
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_DISABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, WARNING_SetAfterSettingLogger) {
    setLogger(mockLogger.get());
    setLevel(WARN);
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_DISABLED();
    EXPECT_WARNING_LOG_ENABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, ERROR_SetBeforeSettingLogger) {
    setLevel(ERR);
    setLogger(mockLogger.get());
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_DISABLED();
    EXPECT_WARNING_LOG_DISABLED();
    EXPECT_ERROR_LOG_ENABLED();
}

TEST_F(LoggingLevelTest, ERROR_SetAfterSettingLogger) {
    setLogger(mockLogger.get());
    setLevel(ERR);
    EXPECT_DEBUG_LOG_DISABLED();
    EXPECT_INFO_LOG_DISABLED();
    EXPECT_WARNING_LOG_DISABLED();
    EXPECT_ERROR_LOG_ENABLED();
}
