#include "testutils/CliTest.h"

using CliTest_ShowingHelp = CliTest;

using cryfs::ErrorCode;

TEST_F(CliTest_ShowingHelp, HelpLongOption) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"--help"}, "", ErrorCode::Success);
}

TEST_F(CliTest_ShowingHelp, HelpLongOptionTogetherWithOtherOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({basedir.string().c_str(), mountdir.string().c_str(), "--help"}, "", ErrorCode::Success);
}

TEST_F(CliTest_ShowingHelp, HelpShortOption) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"-h"}, "", ErrorCode::Success);
}

TEST_F(CliTest_ShowingHelp, HelpShortOptionTogetherWithOtherOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({basedir.string().c_str(), mountdir.string().c_str(), "-h"}, "", ErrorCode::Success);
}

TEST_F(CliTest_ShowingHelp, MissingAllOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({}, "Please specify a base directory", ErrorCode::InvalidArguments);
}

TEST_F(CliTest_ShowingHelp, MissingDir) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({basedir.string().c_str()}, "Please specify a mount directory", ErrorCode::InvalidArguments);
}
