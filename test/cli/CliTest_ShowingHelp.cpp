#include "testutils/CliTest.h"

using CliTest_ShowingHelp = CliTest;

TEST_F(CliTest_ShowingHelp, HelpLongOption) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"--help"});
}

TEST_F(CliTest_ShowingHelp, HelpLongOptionTogetherWithOtherOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({basedir.c_str(), mountdir.c_str(), "--help"});
}

TEST_F(CliTest_ShowingHelp, HelpShortOption) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"-h"});
}

TEST_F(CliTest_ShowingHelp, HelpShortOptionTogetherWithOtherOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({basedir.c_str(), mountdir.c_str(), "-h"});
}

TEST_F(CliTest_ShowingHelp, MissingAllOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({});
}

TEST_F(CliTest_ShowingHelp, MissingDir) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({basedir.c_str()});
}
