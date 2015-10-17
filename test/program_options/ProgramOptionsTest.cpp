#include "testutils/ProgramOptionsTestBase.h"
#include "../../src/program_options/ProgramOptions.h"
#include <messmer/cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using namespace cryfs::program_options;
using std::vector;
using boost::none;
using std::string;

class ProgramOptionsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsTest, BaseDir) {
    ProgramOptions testobj("/home/user/mydir", "", "", false, none, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/mydir", testobj.baseDir());
}

TEST_F(ProgramOptionsTest, MountDir) {
    ProgramOptions testobj("", "/home/user/mydir", "", false, none, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/mydir", testobj.mountDir());
}

TEST_F(ProgramOptionsTest, ConfigFile) {
    ProgramOptions testobj("", "", "/home/user/configfile", false, none, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/configfile", testobj.configFile());
}

TEST_F(ProgramOptionsTest, ForegroundFalse) {
    ProgramOptions testobj("", "", "/home/user/configfile", false, none, options({"./myExecutable"}));
    EXPECT_FALSE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, ForegroundTrue) {
    ProgramOptions testobj("", "", "/home/user/configfile", true, none, options({"./myExecutable"}));
    EXPECT_TRUE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, LogfileNone) {
    ProgramOptions testobj("", "", "/home/user/configfile", true, none, options({"./myExecutable"}));
    EXPECT_EQ(none, testobj.logFile());
}

TEST_F(ProgramOptionsTest, LogfileSome) {
    ProgramOptions testobj("", "", "/home/user/configfile", true, string("logfile"), options({"./myExecutable"}));
    EXPECT_EQ("logfile", testobj.logFile().get());
}

TEST_F(ProgramOptionsTest, EmptyFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", "/home/user/configfile", false, none, options({"./myExecutable"}));
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mydir"}, testobj.fuseOptions());
}

TEST_F(ProgramOptionsTest, SomeFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", "/home/user/configfile", false, none, options({"./myExecutable", "-f", "--longoption"}));
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mydir", "-f", "--longoption"}, testobj.fuseOptions());
}
