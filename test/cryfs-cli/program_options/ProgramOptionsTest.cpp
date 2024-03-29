#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/ProgramOptions.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using namespace cryfs_cli::program_options;
using boost::none;
using boost::optional;
using std::ostream;
using std::string;
namespace bf = boost::filesystem;

// This is needed for google test to work with boost::optional<boost::filesystem::path>
namespace boost {
    template<> inline ostream& operator<< <char, std::char_traits<char>, bf::path>(ostream &stream, const optional<bf::path> &path) {
        if (path == none) {
            return stream << "none";
        }
        return stream << *path;
    }
}

class ProgramOptionsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsTest, BaseDir) {
    const ProgramOptions testobj("/home/user/mydir", "", none, false, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ("/home/user/mydir", testobj.baseDir());
}

TEST_F(ProgramOptionsTest, MountDir) {
    const ProgramOptions testobj("", "/home/user/mydir", none, false, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ("/home/user/mydir", testobj.mountDir());
}

TEST_F(ProgramOptionsTest, ConfigfileNone) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(none, testobj.configFile());
}

TEST_F(ProgramOptionsTest, ConfigfileSome) {
    const ProgramOptions testobj("", "", bf::path("/home/user/configfile"), true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ("/home/user/configfile", testobj.configFile().get());
}

TEST_F(ProgramOptionsTest, ForegroundFalse) {
    const ProgramOptions testobj("", "", none, false, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_FALSE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, ForegroundTrue) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_TRUE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, AllowFilesystemUpgradeFalse) {
    const ProgramOptions testobj("", "", none, false, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_FALSE(testobj.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsTest, AllowFilesystemUpgradeTrue) {
  const ProgramOptions testobj("", "", none, false, true, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_TRUE(testobj.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsTest, CreateMissingBasedirFalse) {
    const ProgramOptions testobj("", "", none, false, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_FALSE(testobj.createMissingBasedir());
}

TEST_F(ProgramOptionsTest, CreateMissingBasedirTrue) {
  const ProgramOptions testobj("", "", none, false, true, false, true, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_TRUE(testobj.createMissingBasedir());
}

TEST_F(ProgramOptionsTest, CreateMissingMountpointFalse) {
    const ProgramOptions testobj("", "", none, false, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_FALSE(testobj.createMissingMountpoint());
}

TEST_F(ProgramOptionsTest, CreateMissingMountpointTrue) {
  const ProgramOptions testobj("", "", none, false, true, false, false, true, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_TRUE(testobj.createMissingMountpoint());
}

TEST_F(ProgramOptionsTest, LogfileNone) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(none, testobj.logFile());
}

TEST_F(ProgramOptionsTest, LogfileSome) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, bf::path("logfile"), none, none, false, none, {"./myExecutable"});
    EXPECT_EQ("logfile", testobj.logFile().get());
}

TEST_F(ProgramOptionsTest, UnmountAfterIdleMinutesNone) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(none, testobj.unmountAfterIdleMinutes());
}

TEST_F(ProgramOptionsTest, UnmountAfterIdleMinutesSome) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, 10, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(10, testobj.unmountAfterIdleMinutes().get());
}

TEST_F(ProgramOptionsTest, CipherNone) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(none, testobj.cipher());
}

TEST_F(ProgramOptionsTest, CipherSome) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, string("aes-256-gcm"), none, false, none, {"./myExecutable"});
    EXPECT_EQ("aes-256-gcm", testobj.cipher().get());
}

TEST_F(ProgramOptionsTest, BlocksizeBytesNone) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(none, testobj.blocksizeBytes());
}

TEST_F(ProgramOptionsTest, BlocksizeBytesSome) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, 10*1024, false, none, {"./myExecutable"});
    EXPECT_EQ(10*1024u, testobj.blocksizeBytes().get());
}

TEST_F(ProgramOptionsTest, MissingBlockIsIntegrityViolationTrue) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, true, {"./myExecutable"});
    EXPECT_TRUE(testobj.missingBlockIsIntegrityViolation().value());
}

TEST_F(ProgramOptionsTest, MissingBlockIsIntegrityViolationFalse) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, false, {"./myExecutable"});
    EXPECT_FALSE(testobj.missingBlockIsIntegrityViolation().value());
}

TEST_F(ProgramOptionsTest, MissingBlockIsIntegrityViolationNone) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_EQ(none, testobj.missingBlockIsIntegrityViolation());
}

TEST_F(ProgramOptionsTest, AllowIntegrityViolationsFalse) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, false, none, {"./myExecutable"});
    EXPECT_FALSE(testobj.allowIntegrityViolations());
}

TEST_F(ProgramOptionsTest, AllowIntegrityViolationsTrue) {
    const ProgramOptions testobj("", "", none, true, false, false, false, false, none, none, none, none, true, none, {"./myExecutable"});
    EXPECT_TRUE(testobj.allowIntegrityViolations());
}

TEST_F(ProgramOptionsTest, EmptyFuseOptions) {
    const ProgramOptions testobj("/rootDir", "/home/user/mydir", none, false, false, false, false, false, none, none, none, none, false, none, {});
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({}, testobj.fuseOptions());
}

TEST_F(ProgramOptionsTest, SomeFuseOptions) {
    const ProgramOptions testobj("/rootDir", "/home/user/mydir", none, false, false, false, false, false, none, none, none, none, false, none, {"-f", "--longoption"});
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"-f", "--longoption"}, testobj.fuseOptions());
}
