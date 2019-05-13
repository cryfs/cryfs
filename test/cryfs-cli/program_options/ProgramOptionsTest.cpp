#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/ProgramOptions.h>
#include <cryfs/impl/filesystem/fsblobstore/utils/TimestampUpdateBehavior.h>
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
    ProgramOptions testobj("/home/user/mydir", "", none, false, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ("/home/user/mydir", testobj.baseDir());
}

TEST_F(ProgramOptionsTest, MountDir) {
    ProgramOptions testobj("", "/home/user/mydir", none, false, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ("/home/user/mydir", testobj.mountDir());
}

TEST_F(ProgramOptionsTest, ConfigfileNone) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(none, testobj.configFile());
}

TEST_F(ProgramOptionsTest, ConfigfileSome) {
    ProgramOptions testobj("", "", bf::path("/home/user/configfile"), true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ("/home/user/configfile", testobj.configFile().get());
}

TEST_F(ProgramOptionsTest, ForegroundFalse) {
    ProgramOptions testobj("", "", none, false, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_FALSE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, ForegroundTrue) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_TRUE(testobj.foreground());
}

TEST_F(ProgramOptionsTest, AllowFilesystemUpgradeFalse) {
    ProgramOptions testobj("", "", none, false, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_FALSE(testobj.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsTest, AllowFilesystemUpgradeTrue) {
  ProgramOptions testobj("", "", none, false, true, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_TRUE(testobj.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsTest, LogfileNone) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(none, testobj.logFile());
}

TEST_F(ProgramOptionsTest, LogfileSome) {
    ProgramOptions testobj("", "", none, true, false, false, none, bf::path("logfile"), none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ("logfile", testobj.logFile().get());
}

TEST_F(ProgramOptionsTest, UnmountAfterIdleMinutesNone) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(none, testobj.unmountAfterIdleMinutes());
}

TEST_F(ProgramOptionsTest, UnmountAfterIdleMinutesSome) {
    ProgramOptions testobj("", "", none, true, false, false, 10, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(10, testobj.unmountAfterIdleMinutes().get());
}

TEST_F(ProgramOptionsTest, CipherNone) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(none, testobj.cipher());
}

TEST_F(ProgramOptionsTest, CipherSome) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, string("aes-256-gcm"), none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ("aes-256-gcm", testobj.cipher().get());
}

TEST_F(ProgramOptionsTest, BlocksizeBytesNone) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(none, testobj.blocksizeBytes());
}

TEST_F(ProgramOptionsTest, BlocksizeBytesSome) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, 10*1024, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(10*1024u, testobj.blocksizeBytes().get());
}

TEST_F(ProgramOptionsTest, MissingBlockIsIntegrityViolationTrue) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, true, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_TRUE(testobj.missingBlockIsIntegrityViolation().value());
}

TEST_F(ProgramOptionsTest, MissingBlockIsIntegrityViolationFalse) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, false, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_FALSE(testobj.missingBlockIsIntegrityViolation().value());
}

TEST_F(ProgramOptionsTest, MissingBlockIsIntegrityViolationNone) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_EQ(none, testobj.missingBlockIsIntegrityViolation());
}

TEST_F(ProgramOptionsTest, AllowIntegrityViolationsFalse) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_FALSE(testobj.allowIntegrityViolations());
}

TEST_F(ProgramOptionsTest, AllowIntegrityViolationsTrue) {
    ProgramOptions testobj("", "", none, true, false, false, none, none, none, none, true, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"./myExecutable"});
    EXPECT_TRUE(testobj.allowIntegrityViolations());
}

TEST_F(ProgramOptionsTest, EmptyFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", none, false, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {});
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({}, testobj.fuseOptions());
}

TEST_F(ProgramOptionsTest, SomeFuseOptions) {
    ProgramOptions testobj("/rootDir", "/home/user/mydir", none, false, false, false, none, none, none, none, false, none, cryfs::fsblobstore::TimestampUpdateBehavior::RELATIME, {"-f", "--longoption"});
    //Fuse should have the mount dir as first parameter
    EXPECT_VECTOR_EQ({"-f", "--longoption"}, testobj.fuseOptions());
}

