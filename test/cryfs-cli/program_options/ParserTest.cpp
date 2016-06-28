#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/Parser.h>
#include <cryfs/config/CryCipher.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using namespace cryfs;
using namespace cryfs::program_options;
using std::vector;
using std::string;
using boost::none;
namespace bf = boost::filesystem;

class ProgramOptionsParserTest: public ProgramOptionsTestBase {
public:
    ProgramOptions parse(std::initializer_list<const char*> options) {
        vector<const char*> _options = options;
        return Parser(_options.size(), _options.data()).parse(CryCiphers::supportedCipherNames());
    }
};

TEST_F(ProgramOptionsParserTest, MissingAllOptions) {
    EXPECT_DEATH(
        parse({"./myExecutable"}),
        "Usage:"
    );
}

TEST_F(ProgramOptionsParserTest, MissingDir) {
    EXPECT_DEATH(
        parse({"./myExecutable", "/home/user/baseDir"}),
        "Usage:"
    );
}

TEST_F(ProgramOptionsParserTest, HelpLongOption) {
    EXPECT_DEATH(
        parse({"./myExecutable", "--help"}),
        "Usage:"
    );
}

TEST_F(ProgramOptionsParserTest, HelpShortOption) {
    EXPECT_DEATH(
        parse({"./myExecutable", "-h"}),
        "Usage:"
    );
}

TEST_F(ProgramOptionsParserTest, ShowCiphers) {
    EXPECT_EXIT(
        parse({"./myExecutable", "--show-ciphers"}),
        ::testing::ExitedWithCode(0),
        "aes-256-gcm"
    );
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
    EXPECT_DEATH(
            parse({"./myExecutable", "/home/user/baseDir", "--cipher", "invalid-cipher", "/home/user/mountDir"}),
            "Invalid cipher: invalid-cipher"
    );
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
