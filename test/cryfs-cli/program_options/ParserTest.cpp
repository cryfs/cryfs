#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/Parser.h>
#include <cryfs/config/CryCipher.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <gitversion/gitversion.h>
#include <cryfs/CryfsException.h>
#include <cpp-utils/testutils/CaptureStderrRAII.h>

using namespace cryfs;
using namespace cryfs::program_options;
using std::vector;
using std::string;
using boost::none;
namespace bf = boost::filesystem;
using cpputils::CaptureStderrRAII;

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
      parse({"./myExecutable", "/home/user/baseDir"});
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
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/baseDir", options.baseDir());
}

TEST_F(ProgramOptionsParserTest, Basedir_Relative) {
    ProgramOptions options = parse({"./myExecutable", "baseDir", "/home/user/mountDir"});
    EXPECT_EQ(bf::current_path() / "baseDir", options.baseDir());
}

TEST_F(ProgramOptionsParserTest, MountDir_Absolute) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/mountDir", options.mountDir());
}

TEST_F(ProgramOptionsParserTest, MountDir_Relative) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "mountDir"});
    EXPECT_EQ(bf::current_path() / "mountDir", options.mountDir());
}

TEST_F(ProgramOptionsParserTest, Foreground_False) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/basedir", "mountdir"});
    EXPECT_FALSE(options.foreground());
}

TEST_F(ProgramOptionsParserTest, Foreground_True) {
    ProgramOptions options = parse({"./myExecutable", "-f", "/home/user/basedir", "mountdir"});
    EXPECT_TRUE(options.foreground());
}

TEST_F(ProgramOptionsParserTest, AllowFilesystemUpgrade_False) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/basedir", "mountdir"});
    EXPECT_FALSE(options.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsParserTest, AllowFilesystemUpgrade_True) {
    ProgramOptions options = parse({"./myExecutable", "--allow-filesystem-upgrade", "/home/user/basedir", "mountdir"});
    EXPECT_TRUE(options.allowFilesystemUpgrade());
}

TEST_F(ProgramOptionsParserTest, LogfileGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--logfile", "/home/user/mylogfile", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/mylogfile", options.logFile().value());
}

TEST_F(ProgramOptionsParserTest, LogfileGiven_RelativePath) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--logfile", "mylogfile", "/home/user/mountDir"});
    EXPECT_EQ(bf::current_path() / "mylogfile", options.logFile().value());
}

TEST_F(ProgramOptionsParserTest, LogfileNotGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ(none, options.logFile());
}

TEST_F(ProgramOptionsParserTest, ConfigfileGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--config", "/home/user/myconfigfile", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/myconfigfile", options.configFile().value());
}

TEST_F(ProgramOptionsParserTest, ConfigfileGiven_RelativePath) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--config", "myconfigfile", "/home/user/mountDir"});
    EXPECT_EQ(bf::current_path() / "myconfigfile", options.configFile().value());
}

TEST_F(ProgramOptionsParserTest, ConfigfileNotGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ(none, options.configFile());
}

TEST_F(ProgramOptionsParserTest, CipherGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--cipher", "aes-256-gcm", "/home/user/mountDir"});
    EXPECT_EQ("aes-256-gcm", options.cipher().value());
}

TEST_F(ProgramOptionsParserTest, CipherNotGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ(none, options.cipher());
}

TEST_F(ProgramOptionsParserTest, InvalidCipher) {
    try {
      parse({"./myExecutable", "/home/user/baseDir", "--cipher", "invalid-cipher", "/home/user/mountDir"});
      EXPECT_TRUE(false); // expect throw
    } catch (const CryfsException& e) {
      EXPECT_EQ(ErrorCode::InvalidArguments, e.errorCode());
      EXPECT_THAT(e.what(), testing::MatchesRegex(".*Invalid cipher: invalid-cipher.*"));
    }
}

TEST_F(ProgramOptionsParserTest, UnmountAfterIdleMinutesGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--unmount-idle", "10", "/home/user/mountDir"});
    EXPECT_EQ(10, options.unmountAfterIdleMinutes().value());
}

TEST_F(ProgramOptionsParserTest, UnmountAfterIdleMinutesGiven_Float) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--unmount-idle", "0.5", "/home/user/mountDir"});
    EXPECT_EQ(0.5, options.unmountAfterIdleMinutes().value());
}

TEST_F(ProgramOptionsParserTest, UnmountAfterIdleMinutesNotGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ(none, options.unmountAfterIdleMinutes());
}

TEST_F(ProgramOptionsParserTest, BlocksizeGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--blocksize", "10240", "/home/user/mountDir"});
    EXPECT_EQ(10240u, options.blocksizeBytes().value());
}

TEST_F(ProgramOptionsParserTest, BlocksizeNotGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ(none, options.blocksizeBytes());
}

TEST_F(ProgramOptionsParserTest, FuseOptionGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir", "--", "-f"});
    EXPECT_EQ("/home/user/baseDir", options.baseDir());
    EXPECT_EQ("/home/user/mountDir", options.mountDir());
    EXPECT_VECTOR_EQ({"-f"}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, FuseOptionGiven_Empty) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir", "--"});
    EXPECT_EQ("/home/user/baseDir", options.baseDir());
    EXPECT_EQ("/home/user/mountDir", options.mountDir());
    EXPECT_VECTOR_EQ({}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, FuseOptionNotGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/baseDir", options.baseDir());
    EXPECT_EQ("/home/user/mountDir", options.mountDir());
    EXPECT_VECTOR_EQ({}, options.fuseOptions());
}
