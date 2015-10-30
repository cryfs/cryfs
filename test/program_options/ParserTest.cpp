#include "testutils/ProgramOptionsTestBase.h"
#include "../../src/program_options/Parser.h"
#include "../../src/config/CryCipher.h"

using namespace cryfs;
using namespace cryfs::program_options;
using std::vector;
using boost::none;

class ProgramOptionsParserTest: public ProgramOptionsTestBase {
public:
    ProgramOptions parse(std::initializer_list<const char*> options) {
        vector<char*> _options = ProgramOptionsTestBase::options(options);
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


TEST_F(ProgramOptionsParserTest, NoSpecialOptions) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/baseDir", options.baseDir());
    EXPECT_EQ("/home/user/mountDir", options.mountDir());
    EXPECT_EQ(none, options.logFile());
    EXPECT_EQ(none, options.configFile());
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mountDir"}, options.fuseOptions());
}

TEST_F(ProgramOptionsParserTest, LogfileGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--logfile", "/home/user/mylogfile", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/mylogfile", options.logFile().value());
}

TEST_F(ProgramOptionsParserTest, ConfigfileGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--config", "/home/user/myconfigfile", "/home/user/mountDir"});
    EXPECT_EQ("/home/user/myconfigfile", options.configFile().value());
}

TEST_F(ProgramOptionsParserTest, CipherGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "--cipher", "aes-256-gcm", "/home/user/mountDir"});
    EXPECT_EQ("aes-256-gcm", options.cipher().value());
}

TEST_F(ProgramOptionsParserTest, InvalidCipher) {
    EXPECT_DEATH(
        parse({"./myExecutable", "/home/user/baseDir", "--cipher", "invalid-cipher", "/home/user/mountDir"}),
        "Invalid cipher: invalid-cipher"
    );
}

TEST_F(ProgramOptionsParserTest, FuseOptionGiven) {
    ProgramOptions options = parse({"./myExecutable", "/home/user/baseDir", "/home/user/mountDir", "--", "-f"});
    EXPECT_EQ("/home/user/baseDir", options.baseDir());
    EXPECT_EQ("/home/user/mountDir", options.mountDir());
    EXPECT_VECTOR_EQ({"./myExecutable", "/home/user/mountDir", "-f"}, options.fuseOptions());
}
