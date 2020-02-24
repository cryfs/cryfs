#include "testutils/CliTest.h"

using cpputils::TempFile;

namespace bf = boost::filesystem;

//Tests that cryfs is correctly setup according to the CLI parameters specified
using CliTest_Setup = CliTest;

TEST_F(CliTest_Setup, NoSpecialOptions) {
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"}, mountdir);
}

TEST_F(CliTest_Setup, NotexistingLogfileGiven) {
    TempFile notexisting_logfile(false);
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountdir.string().c_str(), "-f", "--cipher", "aes-256-gcm", "--logfile", notexisting_logfile.path().string().c_str()}, mountdir);
    //TODO Expect logfile is used (check logfile content)
}

TEST_F(CliTest_Setup, ExistingLogfileGiven) {
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountdir.string().c_str(), "-f", "--cipher", "aes-256-gcm", "--logfile", logfile.path().string().c_str()}, mountdir);
    //TODO Expect logfile is used (check logfile content)
}

TEST_F(CliTest_Setup, ConfigfileGiven) {
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountdir.string().c_str(), "-f", "--cipher", "aes-256-gcm", "--config", configfile.path().string().c_str()}, mountdir);
}

TEST_F(CliTest_Setup, FuseOptionGiven) {
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), mountdir.string().c_str(), "-f", "--cipher", "aes-256-gcm", "--", "-f"}, mountdir);
}

TEST_F(CliTest, WorksWithCommasInBasedir) {
    // This test makes sure we don't regress on https://github.com/cryfs/cryfs/issues/326
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    auto basedir_ = basedir / "pathname,with,commas";
    bf::create_directory(basedir_);
    EXPECT_RUN_SUCCESS({basedir_.string().c_str(), mountdir.string().c_str(), "-f"}, mountdir);
}
