#include "testutils/CliTest.h"

#include <cpp-utils/system/env.h>

namespace bf = boost::filesystem;
using ::testing::Values;
using ::testing::WithParamInterface;
using ::testing::Return;
using std::vector;
using cpputils::TempFile;
using cryfs::ErrorCode;

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
        EXPECT_RUN_SUCCESS(args(), mountdir);
    }

    void Test_Run_Error(const char *expectedError, cryfs::ErrorCode errorCode) {
        EXPECT_RUN_ERROR(
            args(),
            expectedError,
            errorCode
        );
    }

    vector<string> args() {
        vector<string> result = {basedir.string(), mountdir.string()};
        if (GetParam().externalConfigfile) {
            result.push_back("--config");
            result.push_back(configfile.path().string());
        }
        if (GetParam().logIsNotStderr) {
            result.push_back("--logfile");
            result.push_back(logfile.path().string());
        }
        if (GetParam().runningInForeground) {
            result.push_back("-f");
        }
        // Test case should be non-interactive, so don't ask for cipher.
        result.push_back("--cipher");
        result.push_back("aes-256-gcm");
        return result;
    }
};

INSTANTIATE_TEST_SUITE_P(DefaultParams, CliTest_WrongEnvironment, Values(TestConfig({false, false, false})));
INSTANTIATE_TEST_SUITE_P(ExternalConfigfile, CliTest_WrongEnvironment, Values(TestConfig({true, false, false})));
INSTANTIATE_TEST_SUITE_P(LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({false, true, false})));
INSTANTIATE_TEST_SUITE_P(ExternalConfigfile_LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({true, true, false})));
INSTANTIATE_TEST_SUITE_P(RunningInForeground, CliTest_WrongEnvironment, Values(TestConfig({false, false, true})));
INSTANTIATE_TEST_SUITE_P(RunningInForeground_ExternalConfigfile, CliTest_WrongEnvironment, Values(TestConfig({true, false, true})));
INSTANTIATE_TEST_SUITE_P(RunningInForeground_LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({false, true, true})));
INSTANTIATE_TEST_SUITE_P(RunningInForeground_ExternalConfigfile_LogIsNotStderr, CliTest_WrongEnvironment, Values(TestConfig({true, true, true})));

//Counter-Test. Test that it doesn't fail if we call it without an error condition.
TEST_P(CliTest_WrongEnvironment, NoErrorCondition) {
    if (!GetParam().runningInForeground) {return;} // TODO Make this work also if run in background (see CliTest::EXPECT_RUN_SUCCESS)
    Test_Run_Success();
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir) {
    mountdir = basedir;
    Test_Run_Error("Error 18: base directory can't be inside the mount directory", ErrorCode::BaseDirInsideMountDir);
}

bf::path make_relative(const bf::path &path) {
    bf::path result;
    bf::path cwd = bf::current_path();
    for(auto iter = ++cwd.begin(); iter!=cwd.end(); ++iter) {
        result /= "..";
    }
    result /= path.relative_path();
    return result;
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir_MountDirRelative) {
    mountdir = make_relative(basedir);
    Test_Run_Error("Error 18: base directory can't be inside the mount directory", ErrorCode::BaseDirInsideMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir_BaseDirRelative) {
    mountdir = basedir;
    basedir = make_relative(basedir);
    Test_Run_Error("Error 18: base directory can't be inside the mount directory", ErrorCode::BaseDirInsideMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDirIsBaseDir_BothRelative) {
    basedir = make_relative(basedir);
    mountdir = basedir;
    Test_Run_Error("Error 18: base directory can't be inside the mount directory", ErrorCode::BaseDirInsideMountDir);
}

TEST_P(CliTest_WrongEnvironment, BaseDir_DoesntExist) {
    _basedir.remove();
    // ON_CALL and not EXPECT_CALL, because this is a death test (i.e. it is forked) and gmock EXPECT_CALL in fork children don't report to parents.
    ON_CALL(*console, askYesNo("Could not find base directory. Do you want to create it?", testing::_)).WillByDefault(Return(false));
    Test_Run_Error("Error 16: base directory not found", ErrorCode::InaccessibleBaseDir);
}

TEST_P(CliTest_WrongEnvironment, BaseDir_DoesntExist_Noninteractive) {
    _basedir.remove();
    // We can't set an EXPECT_CALL().Times(0), because this is a death test (i.e. it is forked) and gmock EXPECT_CALL in fork children don't report to parents.
    // So we set a default answer that shouldn't crash and check it's not called by checking that it crashes.
    ON_CALL(*console, askYesNo("Could not find base directory. Do you want to create it?", testing::_)).WillByDefault(Return(true));
    cpputils::setenv("CRYFS_FRONTEND", "noninteractive");
    Test_Run_Error("Error 16: base directory not found", ErrorCode::InaccessibleBaseDir);
    cpputils::unsetenv("CRYFS_FRONTEND");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_DoesntExist_Create) {
    if (!GetParam().runningInForeground) {return;} // TODO Make this work also if run in background (see CliTest::EXPECT_RUN_SUCCESS)
    _basedir.remove();
    ON_CALL(*console, askYesNo("Could not find base directory. Do you want to create it?", testing::_)).WillByDefault(Return(true));
    Test_Run_Success();
    EXPECT_TRUE(bf::exists(_basedir.path()) && bf::is_directory(_basedir.path()));
}

TEST_P(CliTest_WrongEnvironment, BaseDir_IsNotDirectory) {
    TempFile basedirfile;
    basedir = basedirfile.path();
    Test_Run_Error("Error 16: base directory is not a directory", ErrorCode::InaccessibleBaseDir);
}

TEST_P(CliTest_WrongEnvironment, BaseDir_AllPermissions) {
    if (!GetParam().runningInForeground) {return;} // TODO Make this work also if run in background (see CliTest::EXPECT_RUN_SUCCESS)
    //Counter-Test. Test it doesn't fail if permissions are there.
    SetAllPermissions(basedir);
    Test_Run_Success();
}

// boost::filesystem doesn't set permissions on Windows correctly
#if !defined(_MSC_VER)
TEST_P(CliTest_WrongEnvironment, BaseDir_NoReadPermission) {
    SetNoReadPermission(basedir);
    Test_Run_Error("Error 16: Could not read from base directory", ErrorCode::InaccessibleBaseDir);
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoExePermission) {
	SetNoExePermission(basedir);
	Test_Run_Error("Error 16: Could not write to base directory", ErrorCode::InaccessibleBaseDir);
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoWritePermission) {
    SetNoWritePermission(basedir);
    Test_Run_Error("Error 16: Could not write to base directory", ErrorCode::InaccessibleBaseDir);
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoPermission) {
    SetNoPermission(basedir);
    Test_Run_Error("Error 16: Could not write to base directory", ErrorCode::InaccessibleBaseDir);
}
#endif

TEST_P(CliTest_WrongEnvironment, MountDir_DoesntExist) {
    _mountdir.remove();
    // ON_CALL and not EXPECT_CALL, because this is a death test (i.e. it is forked) and gmock EXPECT_CALL in fork children don't report to parents.
    ON_CALL(*console, askYesNo("Could not find mount directory. Do you want to create it?", testing::_)).WillByDefault(Return(false));
    Test_Run_Error("mount directory not found", ErrorCode::InaccessibleMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDir_DoesntExist_Noninteractive) {
    _mountdir.remove();
    // We can't set an EXPECT_CALL().Times(0), because this is a death test (i.e. it is forked) and gmock EXPECT_CALL in fork children don't report to parents.
    // So we set a default answer that shouldn't crash and check it's not called by checking that it crashes.
    ON_CALL(*console, askYesNo("Could not find base directory. Do you want to create it?", testing::_)).WillByDefault(Return(true));
    cpputils::setenv("CRYFS_FRONTEND", "noninteractive");
    Test_Run_Error("mount directory not found", ErrorCode::InaccessibleMountDir);
    cpputils::unsetenv("CRYFS_FRONTEND");
}

TEST_P(CliTest_WrongEnvironment, MountDir_DoesntExist_Create) {
    if (!GetParam().runningInForeground) {return;} // TODO Make this work also if run in background (see CliTest::EXPECT_RUN_SUCCESS)
    _mountdir.remove();
    ON_CALL(*console, askYesNo("Could not find mount directory. Do you want to create it?", testing::_)).WillByDefault(Return(true));
    Test_Run_Success();
    EXPECT_TRUE(bf::exists(_mountdir.path()) && bf::is_directory(_mountdir.path()));
}

TEST_P(CliTest_WrongEnvironment, MountDir_IsNotDirectory) {
    TempFile mountdirfile;
    mountdir = mountdirfile.path();
    Test_Run_Error("Error 17: mount directory is not a directory", ErrorCode::InaccessibleMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDir_AllPermissions) {
    if (!GetParam().runningInForeground) {return;} // TODO Make this work also if run in background (see CliTest::EXPECT_RUN_SUCCESS)
    //Counter-Test. Test it doesn't fail if permissions are there.
    SetAllPermissions(mountdir);
    Test_Run_Success();
}

// boost::filesystem doesn't set permissions on Windows correctly
#if !defined(_MSC_VER)
TEST_P(CliTest_WrongEnvironment, MountDir_NoReadPermission) {
    SetNoReadPermission(mountdir);
    Test_Run_Error("Error 17: Could not read from mount directory", ErrorCode::InaccessibleMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDir_NoExePermission) {
	SetNoExePermission(mountdir);
	Test_Run_Error("Error 17: Could not write to mount directory", ErrorCode::InaccessibleMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDir_NoWritePermission) {
    SetNoWritePermission(mountdir);
    Test_Run_Error("Error 17: Could not write to mount directory", ErrorCode::InaccessibleMountDir);
}

TEST_P(CliTest_WrongEnvironment, MountDir_NoPermission) {
    SetNoPermission(mountdir);
    Test_Run_Error("Error 17: Could not write to mount directory", ErrorCode::InaccessibleMountDir);
}
#endif
