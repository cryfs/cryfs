#include "testutils/CliTest.h"

//TODO Test CLI ends with error message (before daemonization), if
// - mountdir does not exist
// - mountdir exists but belongs to other user
// - mountdir exists but is missing permissions
// - TODO when else is libfuse failing? What requirements are there for the mountdir?)
//TODO Test what happens if basedir == mountdir
//TODO Test all stuff (basedir missing, not readable, not writeable, not accessible, ...) also with "-f" foreground flag.


namespace bf = boost::filesystem;

//Tests what happens if cryfs is run in the wrong environment, i.e. with a base directory that doesn't exist or similar
class CliTest_WrongEnvironment: public CliTest {
public:
    void RemoveReadPermission(const bf::path &dir) {
        //TODO Take read permission from basedir in a better way
        system((std::string("chmod -rwx ")+dir.c_str()).c_str());
    }

    void Test_Run_Success() {
        EXPECT_RUN_SUCCESS(
            {basedir.path().c_str(), mountdir.path().c_str()}
        );
    }

    void Test_Run_Error(const char *expectedError) {
        EXPECT_RUN_ERROR(
            {basedir.path().c_str(), mountdir.path().c_str()},
            expectedError
        );
    }

    void Test_Run_LogIsNotStderr_Error(const char *expectedError) {
        //Error message should be shown on stderr, even if a logfile is specified.
        EXPECT_RUN_ERROR(
            {basedir.path().c_str(), mountdir.path().c_str(), "--logfile", logfile.path().c_str()},
            expectedError
        );
    }

    void Test_Run_ExternalConfigfile_Error(const char *expectedError) {
        //Config file writing is one of the first things happening. This test case ensures that even if
        //the config file is not written to the base directory, a wrong base directory is recognized correctly.
        EXPECT_RUN_ERROR(
            {basedir.path().c_str(), mountdir.path().c_str(), "--config", configfile.path().c_str()},
            expectedError
        );
    }

    void Test_Run_ExternalConfigfile_LogIsNotStderr_Error(const char *expectedError) {
        EXPECT_RUN_ERROR(
            {basedir.path().c_str(), mountdir.path().c_str(), "--logfile", logfile.path().c_str(), "--config", configfile.path().c_str()},
            expectedError
        );
    }    
};

TEST_F(CliTest_WrongEnvironment, NoErrorCondition) {
    //Counter-Test. Test that it doesn't fail if we call it without an error condition.
    Test_Run_Success();
}

TEST_F(CliTest_WrongEnvironment, BaseDir_DoesntExist) {
    basedir.remove();
    Test_Run_Error("Error: Base directory not found");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_DoesntExist_LogIsNotStderr) {
    basedir.remove();
    Test_Run_LogIsNotStderr_Error("Error: Base directory not found");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_DoesntExist_ExternalConfigfile) {
    basedir.remove();
    Test_Run_ExternalConfigfile_Error("Error: Base directory not found");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_DoesntExist_ExternalConfigfile_LogIsNotStderr) {
    basedir.remove();
    Test_Run_ExternalConfigfile_LogIsNotStderr_Error("Error: Base directory not found");
}

//TODO finish the following test cases
/*
TEST_F(CliTest_WrongEnvironment, BaseDir_NoReadPermission) {
    RemoveReadPermission(basedir);
    Test_Run_Error("Error: Base directory not readable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoReadPermission_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not readable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoReadPermission_ExternalConfigfile) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not readable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoReadPermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not readable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoWritePermission) {
    RemoveReadPermission(basedir);
    Test_Run_Error("Error: Base directory not writeable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoWritePermission_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not writeable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoWritePermission_ExternalConfigfile) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not writeable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoWritePermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not writeable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoAccessPermission) {
    RemoveAccessPermission(basedir);
    Test_Run_Error("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoAccessPermission_LogIsNotStderr) {
    RemoveAccessPermission(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoAccessPermission_ExternalConfigfile) {
    RemoveAccessPermission(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoAccessPermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveAccessPermission(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoPermission) {
    RemoveAllPermissions(basedir);
    Test_Run_Error("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoPermission_LogIsNotStderr) {
    RemoveAllPermissions(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoPermission_ExternalConfigfile) {
    RemoveAllPermissions(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not accessable");
}

TEST_F(CliTest_WrongEnvironment, BaseDir_NoPermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveAllPermissions(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not accessable");
}

*/
