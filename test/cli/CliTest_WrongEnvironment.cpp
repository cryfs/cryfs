#include "testutils/CliTest.h"

//TODO Test CLI ends with error message (before daemonization), if
// - mountdir does not exist
// - mountdir exists but belongs to other user
// - mountdir exists but is missing permissions
// - TODO when else is libfuse failing? What requirements are there for the mountdir?)
//TODO Test what happens if basedir == mountdir


namespace bf = boost::filesystem;
using ::testing::Values;
using ::testing::WithParamInterface;
using std::vector;

struct TestConfig {
    bool externalConfigfile;
    bool logIsNotStderr;
    bool runningInForeground;
};

//Tests what happens if cryfs is run in the wrong environment, i.e. with a base directory that doesn't exist or similar
class CliTest_WrongEnvironment: public CliTest, public WithParamInterface<TestConfig> {
public:
    void RemoveReadPermission(const bf::path &dir) {
        //TODO Take read permission from basedir in a better way
        system((std::string("chmod -rwx ")+dir.c_str()).c_str());
    }

    void Test_Run_Success() {
        EXPECT_RUN_SUCCESS(args());
    }

    void Test_Run_Error(const char *expectedError) {
        EXPECT_RUN_ERROR(
            args(),
            expectedError
        );
    }

    vector<const char*> args() {
        vector<const char*> result = {basedir.path().c_str(), mountdir.path().c_str()};
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

TEST_P(CliTest_WrongEnvironment, BaseDir_DoesntExist) {
    basedir.remove();
    Test_Run_Error("Error: Base directory not found");
}

//TODO finish the following test cases
/*
TEST_P(CliTest_WrongEnvironment, BaseDir_NoReadPermission) {
    RemoveReadPermission(basedir);
    Test_Run_Error("Error: Base directory not readable");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoWritePermission) {
    RemoveWritePermission(basedir);
    Test_Run_Error("Error: Base directory not writeable");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoAccessPermission) {
    RemoveAccessPermission(basedir);
    Test_Run_Error("Error: Base directory not accessable");
}

TEST_P(CliTest_WrongEnvironment, BaseDir_NoPermission) {
    RemoveAllPermissions(basedir);
    Test_Run_Error("Error: Base directory not accessable");
}

*/
