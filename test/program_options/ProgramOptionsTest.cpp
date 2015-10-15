#include "testutils/ProgramOptionsTestBase.h"
#include "../../src/program_options/ProgramOptions.h"

using namespace cryfs::program_options;
using std::vector;

class ProgramOptionsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsTest, BaseDir) {
    ProgramOptions testobj("/home/user/mydir", "", "", false, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/mydir", testobj.baseDir());
}

TEST_F(ProgramOptionsTest, MountDir) {
    ProgramOptions testobj("", "/home/user/mydir", "", false, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/mydir", testobj.mountDir());
}

TEST_F(ProgramOptionsTest, ConfigFile) {
    ProgramOptions testobj("", "", "/home/user/configfile", false, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/configfile", testobj.configFile());
}

TEST_F(ProgramOptionsTest, ForegroundFalse) {
    ProgramOptions testobj("", "", "/home/user/configfile", false, options({"./myExecutable"}));
    EXPECT_FALSE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, ForegroundTrue) {
    ProgramOptions testobj("", "", "/home/user/configfile", true, options({"./myExecutable"}));
    EXPECT_TRUE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, EmptyFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", "/home/user/configfile", false, options({"./myExecutable"}));
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mydir"}, testobj.fuseOptions());
}

TEST_F(ProgramOptionsTest, SomeFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", "/home/user/configfile", false, options({"./myExecutable", "-f", "--longoption"}));
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mydir", "-f", "--longoption"}, testobj.fuseOptions());
}
