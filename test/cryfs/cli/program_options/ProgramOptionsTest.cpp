#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs/cli/program_options/ProgramOptions.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using namespace cryfs::program_options;
using std::vector;
using boost::none;
using boost::optional;
using std::ostream;
using std::string;
namespace bf = boost::filesystem;

// This is needed for google test to work with boost::optional<boost::filesystem::path>
namespace boost {
    template<> ostream& operator<< <char, std::char_traits<char>, bf::path>(ostream &stream, const optional<bf::path> &path) {
        if (path == none) {
            return stream << "none";
        }
        return stream << *path;
    }
}

class ProgramOptionsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsTest, BaseDir) {
    ProgramOptions testobj("/home/user/mydir", "", none, false, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/mydir", testobj.baseDir());
}

TEST_F(ProgramOptionsTest, MountDir) {
    ProgramOptions testobj("", "/home/user/mydir", none, false, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/mydir", testobj.mountDir());
}

TEST_F(ProgramOptionsTest, ConfigfileNone) {
    ProgramOptions testobj("", "", none, true, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ(none, testobj.configFile());
}

TEST_F(ProgramOptionsTest, ConfigfileSome) {
    ProgramOptions testobj("", "", bf::path("/home/user/configfile"), true, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ("/home/user/configfile", testobj.configFile().get());
}

TEST_F(ProgramOptionsTest, ForegroundFalse) {
    ProgramOptions testobj("", "", none, false, none, none, none, none, options({"./myExecutable"}));
    EXPECT_FALSE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, ForegroundTrue) {
    ProgramOptions testobj("", "", none, true, none, none, none, none, options({"./myExecutable"}));
    EXPECT_TRUE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, LogfileNone) {
    ProgramOptions testobj("", "", none, true, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ(none, testobj.logFile());
}

TEST_F(ProgramOptionsTest, LogfileSome) {
    ProgramOptions testobj("", "", none, true, none, bf::path("logfile"), none, none, options({"./myExecutable"}));
    EXPECT_EQ("logfile", testobj.logFile().get());
}

TEST_F(ProgramOptionsTest, UnmountAfterIdleMinutesNone) {
ProgramOptions testobj("", "", none, true, none, none, none, none, options({"./myExecutable"}));
EXPECT_EQ(none, testobj.unmountAfterIdleMinutes());
}

TEST_F(ProgramOptionsTest, UnmountAfterIdleMinutesSome) {
    ProgramOptions testobj("", "", none, true, 10, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ(10, testobj.unmountAfterIdleMinutes().get());
}

TEST_F(ProgramOptionsTest, CipherNone) {
    ProgramOptions testobj("", "", none, true, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ(none, testobj.cipher());
}

TEST_F(ProgramOptionsTest, CipherSome) {
    ProgramOptions testobj("", "", none, true, none, none, string("aes-256-gcm"), none, options({"./myExecutable"}));
    EXPECT_EQ("aes-256-gcm", testobj.cipher().get());
}

TEST_F(ProgramOptionsTest, ExtPassNone) {
    ProgramOptions testobj("", "", none, true, none, none, none, none, options({"./myExecutable"}));
    EXPECT_EQ(none, testobj.extPass());
}

TEST_F(ProgramOptionsTest, ExtPassSome) {
    ProgramOptions testobj("", "", none, true, none, none, none, string("echo mypassword"), options({"./myExecutable"}));
    EXPECT_EQ("echo mypassword", testobj.extPass().get());
}

TEST_F(ProgramOptionsTest, EmptyFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", none, false, none, none, none, none, options({"./myExecutable"}));
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mydir"}, testobj.fuseOptions());
}

TEST_F(ProgramOptionsTest, SomeFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", none, false, none, none, none, none, options({"./myExecutable", "-f", "--longoption"}));
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mydir", "-f", "--longoption"}, testobj.fuseOptions());
}
