#include "testutils/CliTest.h"

using cpputils::TempFile;
using cryfs::ErrorCode;

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

TEST_F(CliTest_Setup, AutocreateBasedir) {
    TempFile notexisting_basedir(false);
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({notexisting_basedir.path().string().c_str(), mountdir.string().c_str(), "-f", "--cipher", "aes-256-gcm", "--create-missing-basedir"}, mountdir);
}

TEST_F(CliTest_Setup, AutocreateBasedirFail) {
    TempFile notexisting_basedir(false);
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_ERROR(
            {notexisting_basedir.path().string().c_str(), mountdir.string().c_str(), "-f", "--cipher", "aes-256-gcm"},
            "Error 16: base directory not found.",
            ErrorCode::InaccessibleBaseDir
    );
}

TEST_F(CliTest_Setup, AutocreateMountpoint) {
    TempFile notexisting_mountpoint(false);
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_SUCCESS({basedir.string().c_str(), notexisting_mountpoint.path().string().c_str(), "-f", "--cipher", "aes-256-gcm", "--create-missing-mountpoint"}, notexisting_mountpoint.path());
}

TEST_F(CliTest_Setup, AutocreateMountdirFail) {
    TempFile notexisting_mountdir(false);
    //Specify --cipher parameter to make it non-interactive
    //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
    EXPECT_RUN_ERROR(
            {basedir.string().c_str(), notexisting_mountdir.path().string().c_str(), "-f", "--cipher", "aes-256-gcm"},
            "Error 17: mount directory not found.",
            ErrorCode::InaccessibleMountDir
    );
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
