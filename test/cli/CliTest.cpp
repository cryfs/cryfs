#include <google/gtest/gtest.h>
#include "../../src/Cli.h"
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>

using ::testing::Test;
using ::testing::ExitedWithCode;
using cpputils::TempDir;
using cpputils::TempFile;
namespace bf = boost::filesystem;

//TODO Test CLI ends with error message (before daemonization), if
// - mountdir does not exist
// - mountdir exists but belongs to other user
// - mountdir exists but is missing permissions
// - TODO when else is libfuse failing? What requirements are there for the mountdir?)
//TODO Test what happens if basedir == mountdir
//TODO Test all stuff (basedir missing, not readable, not writeable, not accessible, ...) also with "-f" foreground flag.

class CliTest : public Test {
public:
    CliTest(): basedir(), mountdir(), logfile(), configfile(false) {}

    TempDir basedir;
    TempDir mountdir;
    TempFile logfile;
    TempFile configfile;

    void run(std::initializer_list<const char*> args) {
        std::vector<char*> _args;
        _args.reserve(args.size()+1);
        _args.push_back(const_cast<char*>("cryfs"));
        for (const char *arg : args) {
            _args.push_back(const_cast<char*>(arg));
        }
        cryfs::Cli().main(_args.size(), _args.data());
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(std::initializer_list<const char*> args) {
        EXPECT_RUN_ERROR(args, "Usage");
    }

    void EXPECT_RUN_ERROR(std::initializer_list<const char*> args, const char *message) {
        EXPECT_EXIT(
            run(args),
            ExitedWithCode(1),
            message
        );
    }

    void EXPECT_RUN_SUCCESS(std::initializer_list<const char*> args) {
        //TODO
        /*EXPECT_EXIT(
            run(args),
            ExitedWithCode(0),
            "Filesystem is running"
        );*/
        //TODO Then stop running cryfs process again
    }

    void RemoveReadPermission(const bf::path &dir) {
        //TODO Take read permission from basedir in a better way
        system((string("chmod -rwx ")+basedir.path().c_str()).c_str());
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

TEST_F(CliTest, HelpLongOption) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"--help"});
}

TEST_F(CliTest, HelpLongOptionTogetherWithOtherOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"/", "/mountdir", "--help"});
}

TEST_F(CliTest, HelpShortOption) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"-h"});
}

TEST_F(CliTest, HelpShortOptionTogetherWithOtherOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"/", "/mountdir", "-h"});
}

TEST_F(CliTest, MissingAllOptions) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({});
}

TEST_F(CliTest, MissingDir) {
    EXPECT_EXIT_WITH_HELP_MESSAGE({"/"});
}

TEST_F(CliTest, NoSpecialOptions) {
    EXPECT_RUN_SUCCESS({basedir.path().c_str(), mountdir.path().c_str()});
}

TEST_F(CliTest, NotexistingLogfileGiven) {
    TempFile notexisting_logfile(false);
    EXPECT_RUN_SUCCESS({basedir.path().c_str(), mountdir.path().c_str(), "--logfile", notexisting_logfile.path().c_str()});
    //TODO Expect logfile is used (check logfile content)
}

TEST_F(CliTest, ExistingLogfileGiven) {
    EXPECT_RUN_SUCCESS({basedir.path().c_str(), mountdir.path().c_str(), "--logfile", logfile.path().c_str()});
    //TODO Expect logfile is used (check logfile content)
}

TEST_F(CliTest, ConfigfileGiven) {
    EXPECT_RUN_SUCCESS({"/home/user/baseDir", "--config", configfile.path().c_str(), "/home/user/mountDir"});
}

TEST_F(CliTest, FuseOptionGiven) {
    EXPECT_RUN_SUCCESS({"/home/user/baseDir", "/home/user/mountDir", "--", "-f"});
}

TEST_F(CliTest, BaseDir_DoesntExist) {
    basedir.remove();
    Test_Run_Error("Error: Base directory not found");
}

TEST_F(CliTest, BaseDir_DoesntExist_LogIsNotStderr) {
    basedir.remove();
    Test_Run_LogIsNotStderr_Error("Error: Base directory not found");
}

TEST_F(CliTest, BaseDir_DoesntExist_ExternalConfigfile) {
    basedir.remove();
    Test_Run_ExternalConfigfile_Error("Error: Base directory not found");
}

TEST_F(CliTest, BaseDir_DoesntExist_ExternalConfigfile_LogIsNotStderr) {
    basedir.remove();
    Test_Run_ExternalConfigfile_LogIsNotStderr_Error("Error: Base directory not found");
}

//TODO finish the following test cases
/*
TEST_F(CliTest, BaseDir_NoReadPermission) {
    RemoveReadPermission(basedir);
    Test_Run_Error("Error: Base directory not readable");
}

TEST_F(CliTest, BaseDir_NoReadPermission_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not readable");
}

TEST_F(CliTest, BaseDir_NoReadPermission_ExternalConfigfile) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not readable");
}

TEST_F(CliTest, BaseDir_NoReadPermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not readable");
}

TEST_F(CliTest, BaseDir_NoWritePermission) {
    RemoveReadPermission(basedir);
    Test_Run_Error("Error: Base directory not writeable");
}

TEST_F(CliTest, BaseDir_NoWritePermission_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not writeable");
}

TEST_F(CliTest, BaseDir_NoWritePermission_ExternalConfigfile) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not writeable");
}

TEST_F(CliTest, BaseDir_NoWritePermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveReadPermission(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not writeable");
}

TEST_F(CliTest, BaseDir_NoAccessPermission) {
    RemoveAccessPermission(basedir);
    Test_Run_Error("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoAccessPermission_LogIsNotStderr) {
    RemoveAccessPermission(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoAccessPermission_ExternalConfigfile) {
    RemoveAccessPermission(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoAccessPermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveAccessPermission(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoPermission) {
    RemoveAllPermissions(basedir);
    Test_Run_Error("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoPermission_LogIsNotStderr) {
    RemoveAllPermissions(basedir);
    Test_Run_LogIsNotStderr_Error("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoPermission_ExternalConfigfile) {
    RemoveAllPermissions(basedir);
    Test_Run_ExternalConfigfile_Error("Error: Base directory not accessable");
}

TEST_F(CliTest, BaseDir_NoPermission_ExternalConfigfile_LogIsNotStderr) {
    RemoveAllPermissions(basedir);
    Test_Run_ExternalConfigfile_LogIsNotStderrError("Error: Base directory not accessable");
}

*/
