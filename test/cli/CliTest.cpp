#include <google/gtest/gtest.h>
#include "../../src/Cli.h"

using ::testing::Test;
using ::testing::ExitedWithCode;

//TODO Test CLI ends with error message (before daemonization), if
// - root dir doesn't exist with configfile inside/outside of rootdir
// - root dir exists, but is missing "r"/"x"/"w"/"rwx" permission, with configfile inside/outside of rootdir
// - mountdir does not exist
// - mountdir exists but belongs to other user
// - mountdir exists but is missing permissions
// - TODO when else is libfuse failing? What requirements are there for the mountdir?)
//TODO Take some test cases from command line options parser and test it is showing help message

class CliTest : public Test {
public:
    void run(std::initializer_list<const char*> args) {
        std::vector<const char*> _args(args);
        cryfs::Cli().main(_args.size(), const_cast<char**>(_args.data()));
    }
};

TEST_F(CliTest, HelpMessage) {
    EXPECT_EXIT(
        run({"--help"}),
        ExitedWithCode(1),
        "Usage"
    );
}
