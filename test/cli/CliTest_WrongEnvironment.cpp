#include "testutils/CliTest.h"

//TODO Test CLI ends with error message (before daemonization), if
// - mountdir does not exist
// - mountdir exists but belongs to other user
// - mountdir exists but is missing permissions
// - TODO when else is libfuse failing? What requirements are there for the mountdir?)


namespace bf = boost::filesystem;
using ::testing::Values;
using ::testing::WithParamInterface;
using std::vector;
using cpputils::TempFile;

struct TestConfig {
    bool externalConfigfile;
    bool logIsNotStderr;
    bool runningInForeground;
};

//Tests what happens if cryfs is run in the wrong environment, i.e. with a base directory that doesn't exist or similar
class CliTest_WrongEnvironment: public CliTest, public WithParamInterface<TestConfig> {
public:
    void SetAllPermissions(const bf::path &dir) {
        bf::permissions(dir, bf::owner_write|bf::owner_read|bf::owner_exe);
    }

    void SetNoReadPermission(const bf::path &dir) {
        bf::permissions(dir, bf::owner_write|bf::owner_exe);
    }

    void SetNoWritePermission(const bf::path &dir) {
        bf::permissions(dir, bf::owner_read|bf::owner_exe);
    }

    void SetNoExePermission(const bf::path &dir) {
        bf::permissions(dir, bf::owner_read|bf::owner_write);
    }

    void SetNoPermission(const bf::path &dir) {
        bf::permissions(dir, bf::no_perms);
    }

    void Test_Run_Success() {
        EXPECT_RUN_SUCCESS(args(), basedir);
    }

    void Test_Run_Error(const char *expectedError) {
        EXPECT_RUN_ERROR(
            args(),
            expectedError
        );
    }

    vector<const char*> args() {
        vector<const char*> result = {basedir.c_str(), mountdir.c_str()};
        if (GetParam().externalConfigfile) {
            result.push_back("--config");
            result.push_back(configfile.path().c_str());
        }
        if (GetParam().logIsNotStderr) {
            result.push_back("--logfile");
            result.push_back(logfile.path().c_str());
        }
        if (GetParam().runningInForeground) {
            result.push_back("-f");
        }
        return result;
    }
};

INSTANTIATE_TEST_CASE_P(DefaultParams, CliTest_WrongEnvironment, Values(TestConfig({false, false, false})));
INSTANTIATE_TEST_CASE_P(ExternalConfigfile, CliTest_WrongEnvironment, Values(TestConfig({true, false, false})));
INSTANTIATE_TEST_CASE_P(LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({false, true, false})));
INSTANTIATE_TEST_CASE_P(ExternalConfigfile_LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({true, true, false})));
INSTANTIATE_TEST_CASE_P(RunningInForeground, CliTest_WrongEnvironment, Values(TestConfig({false, false, true})));
INSTANTIATE_TEST_CASE_P(RunningInForeground_ExternalConfigfile, CliTest_WrongEnvironment, Values(TestConfig({true, false, true})));
INSTANTIATE_TEST_CASE_P(RunningInForeground_LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({false, true, true})));
INSTANTIATE_TEST_CASE_P(RunningInForeground_ExternalConfigfile_LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({true, true, true})));

//Counter-Test. Test that it doesn't fail if we call it without an error condition.
TEST_P(CliTest_WrongEnvironment, NoErrorCondition) {
    Test_Run_Success();
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir) {
    mountdir = basedir;
    Test_Run_Error("Error: Base directory can't be inside the mount directory");
}

bf::path make_relative(const bf::path &path) {
    bf::path result;
    bf::path cwd = bf::current_path();
    for(auto iter = ++cwd.begin(); iter!=cwd.end(); ++iter) {
        result /= "..";
    }
    result /= path;
    return result;
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir_MountDirRelative) {
    mountdir = make_relative(basedir);
    Test_Run_Error("Error: Base directory can't be inside the mount directory");
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir_BaseDirRelative) {
    mountdir = basedir;
    basedir = make_relative(basedir);
    Test_Run_Error("Error: Base directory can't be inside the mount directory");
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir_BothRelative) {
    basedir = make_relative(basedir);
    mountdir = basedir;
    Test_Run_Error("Error: Base directory can't be inside the mount directory");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_DoesntExist) {
    _basedir.remove();
    Test_Run_Error("Error: Base directory not found");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_IsNotDirectory) {
    TempFile basedirfile;
    basedir = basedirfile.path();
    Test_Run_Error("Error: Base directory is not a directory");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_AllPermissions) {
    //Counter-Test. Test it doesn't fail if permissions are there.
    SetAllPermissions(basedir);
    Test_Run_Success();
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoReadPermission) {
    SetNoReadPermission(basedir);
    Test_Run_Error("Error: Could not read from base directory");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoWritePermission) {
    SetNoWritePermission(basedir);
    Test_Run_Error("Error: Could not write to base directory");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoExePermission) {
    SetNoExePermission(basedir);
    Test_Run_Error("Error: Could not write to base directory");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoPermission) {
    SetNoPermission(basedir);
    Test_Run_Error("Error: Could not write to base directory");
}
