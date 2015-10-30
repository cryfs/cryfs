#include "testutils/CliTest.h"

using cpputils::TempFile;

//Tests that cryfs is correctly setup according to the CLI parameters specified
using CliTest_Setup = CliTest;

TEST_F(CliTest_Setup, NoSpecialOptions) {
    EXPECT_RUN_SUCCESS({basedir.c_str(), mountdir.c_str()});
}

TEST_F(CliTest_Setup, NotexistingLogfileGiven) {
    TempFile notexisting_logfile(false);
    EXPECT_RUN_SUCCESS({basedir.c_str(), mountdir.c_str(), "--logfile", notexisting_logfile.path().c_str()});
    //TODO Expect logfile is used (check logfile content)
}

TEST_F(CliTest_Setup, ExistingLogfileGiven) {
    EXPECT_RUN_SUCCESS({basedir.c_str(), mountdir.c_str(), "--logfile", logfile.path().c_str()});
    //TODO Expect logfile is used (check logfile content)
}

TEST_F(CliTest_Setup, ConfigfileGiven) {
    EXPECT_RUN_SUCCESS({basedir.c_str(), "--config", configfile.path().c_str(), mountdir.c_str()});
}

TEST_F(CliTest_Setup, FuseOptionGiven) {
    EXPECT_RUN_SUCCESS({basedir.c_str(), mountdir.c_str(), "--", "-f"});
}