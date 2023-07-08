#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/Parser.h>
#include <cryfs/impl/config/CryCipher.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <gitversion/gitversion.h>
#include <cryfs/impl/CryfsException.h>
#include <cpp-utils/testutils/CaptureStderrRAII.h>

using namespace cryfs;
using namespace cryfs_cli::program_options;
using std::vector;
using std::string;
using boost::none;
namespace bf = boost::filesystem;
using cpputils::CaptureStderrRAII;

#if !defined(_MSC_VER)
constexpr const char *basedir = "/home/user/baseDir";
constexpr const char *mountdir = "/home/user/mountDir";
constexpr const char *logfile = "/home/user/logfile";
constexpr const char *configfile = "/home/user/configfile";
#else
constexpr const char *basedir = "C:\\basedir";
constexpr const char *mountdir = "C:\\mountdir";
constexpr const char *logfile = "C:\\logfile";
constexpr const char *configfile = "C:\\configfile";
#endif

class ProgramOptionsParserTest: public ProgramOptionsTestBase {
public:
    ProgramOptions parse(std::initializer_list<const char*> options) {
        vector<const char*> _options = options;
        return Parser(_options.size(), _options.data()).parse(CryCiphers::supportedCipherNames());
    }
};

TEST_F(ProgramOptionsParserTest, MissingAllOptions) {
    CaptureStderrRAII captureStderr;
    try {
      parse({"./myExecutable"});
      EXPECT_TRUE(false); // expect throws
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::InvalidArguments, e.errorCode());
      captureStderr.EXPECT_MATCHES("Usage:"); // expect show usage information
    }
}

TEST_F(ProgramOptionsParserTest, MissingDir) {
    CaptureStderrRAII captureStderr;
    try {
      parse({"./myExecutable", basedir});
      EXPECT_TRUE(false); // expect throw
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::InvalidArguments, e.errorCode());
      captureStderr.EXPECT_MATCHES("Usage:"); // expect show usage information
    }
}

TEST_F(ProgramOptionsParserTest, HelpLongOption) {
    CaptureStderrRAII captureStderr;
    try {
      parse({"./myExecutable", "--help"});
      EXPECT_TRUE(false); // expect throw
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::Success, e.errorCode());
      captureStderr.EXPECT_MATCHES("Usage:"); // expect show usage information
    }
}

TEST_F(ProgramOptionsParserTest, HelpShortOption) {
    CaptureStderrRAII captureStderr;
    try {
      parse({"./myExecutable", "-h"});
      EXPECT_TRUE(false); // expect throw
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::Success, e.errorCode());
      captureStderr.EXPECT_MATCHES("Usage:"); // expect show usage information
    }
}

TEST_F(ProgramOptionsParserTest, ShowCiphers) {
    CaptureStderrRAII captureStderr;
    try {
      parse({"./myExecutable", "--show-ciphers"});
      EXPECT_TRUE(false); // expect throw
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::Success, e.errorCode());
      captureStderr.EXPECT_MATCHES("aes-256-gcm"); // expect show ciphers
    }
}

TEST_F(ProgramOptionsParserTest, BaseDir_Absolute) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(basedir, options.baseDir());
}

TEST_F(ProgramOptionsParserTest, Basedir_Relative) {
    const ProgramOptions options = parse({"./myExecutable", "baseDir", mountdir});
    EXPECT_EQ(bf::current_path() / "baseDir", options.baseDir());
}

TEST_F(ProgramOptionsParserTest, MountDir_Absolute) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(mountdir, options.mountDir());
}

TEST_F(ProgramOptionsParserTest, MountDir_Relative) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "mountDir"});
    EXPECT_EQ(bf::current_path() / "mountDir", options.mountDir());
}

TEST_F(ProgramOptionsParserTest, Foreground_False) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "mountdir"});
    EXPECT_FALSE(options.foreground());
}

TEST_F(ProgramOptionsParserTest, Foreground_True) {
    const ProgramOptions options = parse({"./myExecutable", "-f", basedir, "mountdir"});
    EXPECT_TRUE(options.foreground());
}

TEST_F(ProgramOptionsParserTest, AllowFilesystemUpgrade_False) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "mountdir"});
    EXPECT_FALSE(options.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsParserTest, AllowFilesystemUpgrade_True) {
    const ProgramOptions options = parse({"./myExecutable", "--allow-filesystem-upgrade", basedir, "mountdir"});
    EXPECT_TRUE(options.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsParserTest, CreateMissingBasedir_False) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "mountdir"});
    EXPECT_FALSE(options.createMissingBasedir());
}

TEST_F(ProgramOptionsParserTest, CreateMissingBasedir_True) {
    const ProgramOptions options = parse({"./myExecutable", "--create-missing-basedir",  basedir, "mountdir"});
    EXPECT_TRUE(options.createMissingBasedir());
}

TEST_F(ProgramOptionsParserTest, CreateMissingMountpoint_False) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "mountdir"});
    EXPECT_FALSE(options.createMissingMountpoint());
}

TEST_F(ProgramOptionsParserTest, CreateMissingMountpoint_True) {
    const ProgramOptions options = parse({"./myExecutable", "--create-missing-mountpoint",  basedir, "mountdir"});
    EXPECT_TRUE(options.createMissingMountpoint());
}

TEST_F(ProgramOptionsParserTest, LogfileGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--logfile", logfile, mountdir});
    EXPECT_EQ(logfile, options.logFile().value());
}

TEST_F(ProgramOptionsParserTest, LogfileGiven_RelativePath) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--logfile", "mylogfile", mountdir});
    EXPECT_EQ(bf::current_path() / "mylogfile", options.logFile().value());
}

TEST_F(ProgramOptionsParserTest, LogfileNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(none, options.logFile());
}

TEST_F(ProgramOptionsParserTest, ConfigfileGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--config", configfile, mountdir});
    EXPECT_EQ(configfile, options.configFile().value());
}

TEST_F(ProgramOptionsParserTest, ConfigfileGiven_RelativePath) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--config", "myconfigfile", mountdir});
    EXPECT_EQ(bf::current_path() / "myconfigfile", options.configFile().value());
}

TEST_F(ProgramOptionsParserTest, ConfigfileNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(none, options.configFile());
}

TEST_F(ProgramOptionsParserTest, CipherGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--cipher", "aes-256-gcm", mountdir});
    EXPECT_EQ("aes-256-gcm", options.cipher().value());
}

TEST_F(ProgramOptionsParserTest, CipherNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(none, options.cipher());
}

TEST_F(ProgramOptionsParserTest, InvalidCipher) {
    try {
      parse({"./myExecutable", basedir, "--cipher", "invalid-cipher", mountdir});
      EXPECT_TRUE(false); // expect throw
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::InvalidArguments, e.errorCode());
      EXPECT_THAT(e.what(), testing::MatchesRegex(".*Invalid cipher: invalid-cipher.*"));
    }
}

TEST_F(ProgramOptionsParserTest, UnmountAfterIdleMinutesGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--unmount-idle", "10", mountdir});
    EXPECT_EQ(10, options.unmountAfterIdleMinutes().value());
}

TEST_F(ProgramOptionsParserTest, UnmountAfterIdleMinutesGiven_Float) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--unmount-idle", "0.5", mountdir});
    EXPECT_EQ(0.5, options.unmountAfterIdleMinutes().value());
}

TEST_F(ProgramOptionsParserTest, UnmountAfterIdleMinutesNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(none, options.unmountAfterIdleMinutes());
}

TEST_F(ProgramOptionsParserTest, BlocksizeGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--blocksize", "10240", mountdir});
    EXPECT_EQ(10240u, options.blocksizeBytes().value());
}

TEST_F(ProgramOptionsParserTest, BlocksizeNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(none, options.blocksizeBytes());
}

TEST_F(ProgramOptionsParserTest, MissingBlockIsIntegrityViolationGiven_True) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--missing-block-is-integrity-violation", "true", mountdir});
    EXPECT_TRUE(options.missingBlockIsIntegrityViolation().value());
}

TEST_F(ProgramOptionsParserTest, MissingBlockIsIntegrityViolationGiven_False) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--missing-block-is-integrity-violation", "false", mountdir});
    EXPECT_FALSE(options.missingBlockIsIntegrityViolation().value());
}

TEST_F(ProgramOptionsParserTest, AllowIntegrityViolations_True) {
    const ProgramOptions options = parse({"./myExecutable", basedir, "--allow-integrity-violations", mountdir});
    EXPECT_TRUE(options.allowIntegrityViolations());
}

TEST_F(ProgramOptionsParserTest, AllowIntegrityViolations_False) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_FALSE(options.allowIntegrityViolations());
}

TEST_F(ProgramOptionsParserTest, MissingBlockIsIntegrityViolationNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(none, options.missingBlockIsIntegrityViolation());
}

TEST_F(ProgramOptionsParserTest, FuseOptionGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir, "--", "-f"});
    EXPECT_EQ(basedir, options.baseDir());
    EXPECT_EQ(mountdir, options.mountDir());
    EXPECT_VECTOR_EQ({"-f"}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, FuseOptionGiven_Empty) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir, "--"});
    EXPECT_EQ(basedir, options.baseDir());
    EXPECT_EQ(mountdir, options.mountDir());
    EXPECT_VECTOR_EQ({}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, FuseOptionNotGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir});
    EXPECT_EQ(basedir, options.baseDir());
    EXPECT_EQ(mountdir, options.mountDir());
    EXPECT_VECTOR_EQ({}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, DirectFuseOptionsGiven_AfterPositionalOptions) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir, "-o", "my_opt"});
    EXPECT_VECTOR_EQ({"-o", "my_opt"}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, DirectFuseOptionsGiven_BeforePositionalOptions) {
    const ProgramOptions options = parse({"./myExecutable", "-o", "my_opt", basedir, mountdir});
    EXPECT_VECTOR_EQ({"-o", "my_opt"}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, DirectFuseOptionsGiven_BeforeAndAfterPositionalOptions) {
    const ProgramOptions options = parse({"./myExecutable", "-o", "first", "-o", "second", basedir, "-o", "third", "-o", "fourth", mountdir, "-o", "fifth", "-o", "sixth"});
    EXPECT_VECTOR_EQ({"-o", "first", "-o", "second", "-o", "third", "-o", "fourth", "-o", "fifth", "-o", "sixth"}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, DirectAndIndirectFuseOptionsGiven) {
    const ProgramOptions options = parse({"./myExecutable", basedir, mountdir, "-o", "my_opt", "--", "-o", "other_opt"});
    EXPECT_VECTOR_EQ({"-o", "other_opt", "-o", "my_opt"}, options.fuseOptions());
}
