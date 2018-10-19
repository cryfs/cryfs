#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cryfs/config/CryConfigCreator.h>
#include <cryfs/config/CryCipher.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "../testutils/MockConsole.h"
#include "../testutils/TestWithFakeHomeDirectory.h"
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <gitversion/gitversion.h>
#include <cryfs/localstate/LocalStateDir.h>

using namespace cryfs;

using boost::none;
using cpputils::NoninteractiveConsole;
using std::string;
using std::shared_ptr;
using std::make_shared;
using ::testing::_;
using ::testing::Return;
using ::testing::HasSubstr;
using ::testing::UnorderedElementsAreArray;
using ::testing::NiceMock;

#define EXPECT_ASK_TO_USE_DEFAULT_SETTINGS()                                                                           \
  EXPECT_CALL(*console, askYesNo("Use default settings?", true)).Times(1)
#define EXPECT_DOES_NOT_ASK_TO_USE_DEFAULT_SETTINGS()                                                                  \
  EXPECT_CALL(*console, askYesNo("Use default settings?", true)).Times(0)
#define EXPECT_ASK_FOR_CIPHER()                                                                                        \
  EXPECT_CALL(*console, ask(HasSubstr("block cipher"), UnorderedElementsAreArray(CryCiphers::supportedCipherNames()))).Times(1)
#define EXPECT_DOES_NOT_ASK_FOR_CIPHER()                                                                               \
  EXPECT_CALL(*console, ask(HasSubstr("block cipher"), _)).Times(0)
#define EXPECT_ASK_FOR_BLOCKSIZE()                                                                                     \
  EXPECT_CALL(*console, ask(HasSubstr("block size"), _)).Times(1)
#define EXPECT_DOES_NOT_ASK_FOR_BLOCKSIZE()                                                                            \
  EXPECT_CALL(*console, ask(HasSubstr("block size"), _)).Times(0)
#define EXPECT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION()                                                              \
  EXPECT_CALL(*console, askYesNo(HasSubstr("missing block"), false)).Times(1)
#define EXPECT_DOES_NOT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION()                                                     \
  EXPECT_CALL(*console, askYesNo(HasSubstr("missing block"), false)).Times(0)
#define IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION()                                                              \
  EXPECT_CALL(*console, askYesNo(HasSubstr("missing block"), false))

class CryConfigCreatorTest: public ::testing::Test, TestWithFakeHomeDirectory {
public:
    CryConfigCreatorTest()
            : console(make_shared<NiceMock<MockConsole>>()),
              tempLocalStateDir(), localStateDir(tempLocalStateDir.path()),
              creator(console, cpputils::Random::PseudoRandom(), localStateDir),
              noninteractiveCreator(make_shared<NoninteractiveConsole>(console), cpputils::Random::PseudoRandom(), localStateDir) {
        EXPECT_CALL(*console, ask(HasSubstr("block cipher"), _)).WillRepeatedly(ChooseAnyCipher());
        EXPECT_CALL(*console, ask(HasSubstr("block size"), _)).WillRepeatedly(Return(0));
    }
    shared_ptr<NiceMock<MockConsole>> console;
    cpputils::TempDir tempLocalStateDir;
    LocalStateDir localStateDir;
    CryConfigCreator creator;
    CryConfigCreator noninteractiveCreator;

    void AnswerNoToDefaultSettings() {
        EXPECT_ASK_TO_USE_DEFAULT_SETTINGS().WillOnce(Return(false));
    }

    void AnswerYesToDefaultSettings() {
        EXPECT_ASK_TO_USE_DEFAULT_SETTINGS().WillOnce(Return(true));
    }
};

TEST_F(CryConfigCreatorTest, DoesAskForCipherIfNotSpecified) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseAnyCipher());
    CryConfig config = creator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskForCipherIfSpecified) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_DOES_NOT_ASK_FOR_CIPHER();
    CryConfig config = creator.create(string("aes-256-gcm"), none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskForCipherIfUsingDefaultSettings) {
    AnswerYesToDefaultSettings();
    EXPECT_DOES_NOT_ASK_FOR_CIPHER();
    CryConfig config = creator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskForCipherIfNoninteractive) {
    EXPECT_DOES_NOT_ASK_TO_USE_DEFAULT_SETTINGS();
    EXPECT_DOES_NOT_ASK_FOR_CIPHER();
    CryConfig config = noninteractiveCreator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesAskForBlocksizeIfNotSpecified) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_ASK_FOR_BLOCKSIZE().WillOnce(Return(1));
    CryConfig config = creator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskForBlocksizeIfSpecified) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_DOES_NOT_ASK_FOR_BLOCKSIZE();
    CryConfig config = creator.create(none, 10*1024u, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskForBlocksizeIfNoninteractive) {
    EXPECT_DOES_NOT_ASK_TO_USE_DEFAULT_SETTINGS();
    EXPECT_DOES_NOT_ASK_FOR_BLOCKSIZE();
    CryConfig config = noninteractiveCreator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskForBlocksizeIfUsingDefaultSettings) {
    AnswerYesToDefaultSettings();
    EXPECT_DOES_NOT_ASK_FOR_BLOCKSIZE();
    CryConfig config = creator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesAskWhetherMissingBlocksAreIntegrityViolationsIfNotSpecified) {
    AnswerNoToDefaultSettings();
    EXPECT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION().WillOnce(Return(true));
    CryConfig config = creator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskWhetherMissingBlocksAreIntegrityViolationsIfSpecified_True) {
    AnswerNoToDefaultSettings();
    EXPECT_DOES_NOT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    CryConfig config = creator.create(none, none, true, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskWhetherMissingBlocksAreIntegrityViolationsIfSpecified_False) {
    AnswerNoToDefaultSettings();
    EXPECT_DOES_NOT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    CryConfig config = creator.create(none, none, false, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskWhetherMissingBlocksAreIntegrityViolationsIfNoninteractive) {
    EXPECT_DOES_NOT_ASK_TO_USE_DEFAULT_SETTINGS();
    EXPECT_DOES_NOT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    CryConfig config = noninteractiveCreator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, DoesNotAskWhetherMissingBlocksAreIntegrityViolationsIfUsingDefaultSettings) {
    AnswerYesToDefaultSettings();
    EXPECT_DOES_NOT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    CryConfig config = creator.create(none, none, none, false).config;
}

TEST_F(CryConfigCreatorTest, ChoosesEmptyRootBlobId) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    CryConfig config = creator.create(none, none, none, false).config;
    EXPECT_EQ("", config.RootBlob()); // This tells CryFS to create a new root blob
}

#if CRYPTOPP_VERSION != 564
TEST_F(CryConfigCreatorTest, ChoosesValidEncryptionKey_448) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher("mars-448-gcm"));
    CryConfig config = creator.create(none, none, none, false).config;
    cpputils::Mars448_GCM::EncryptionKey::FromString(config.EncryptionKey()); // This crashes if invalid
}
#endif

TEST_F(CryConfigCreatorTest, ChoosesValidEncryptionKey_256) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher("aes-256-gcm"));
    CryConfig config = creator.create(none, none, none, false).config;
    cpputils::AES256_GCM::EncryptionKey::FromString(config.EncryptionKey()); // This crashes if invalid
}

TEST_F(CryConfigCreatorTest, ChoosesValidEncryptionKey_128) {
    AnswerNoToDefaultSettings();
    IGNORE_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION();
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher("aes-128-gcm"));
    CryConfig config = creator.create(none, none, none, false).config;
    cpputils::AES128_GCM::EncryptionKey::FromString(config.EncryptionKey()); // This crashes if invalid
}

TEST_F(CryConfigCreatorTest, DoesNotAskForAnythingIfEverythingIsSpecified) {
    EXPECT_DOES_NOT_ASK_TO_USE_DEFAULT_SETTINGS();
    EXPECT_DOES_NOT_ASK_FOR_CIPHER();
    CryConfig config = noninteractiveCreator.create(string("aes-256-gcm"), 10*1024u, none, false).config;
}

TEST_F(CryConfigCreatorTest, SetsCorrectCreatedWithVersion) {
    CryConfig config = noninteractiveCreator.create(none, none, none, false).config;
    EXPECT_EQ(gitversion::VersionString(), config.CreatedWithVersion());
}

TEST_F(CryConfigCreatorTest, SetsCorrectLastOpenedWithVersion) {
    CryConfig config = noninteractiveCreator.create(none, none, none, false).config;
    EXPECT_EQ(gitversion::VersionString(), config.CreatedWithVersion());
}

TEST_F(CryConfigCreatorTest, SetsCorrectVersion) {
    CryConfig config = noninteractiveCreator.create(none, none, none, false).config;
    EXPECT_EQ(CryConfig::FilesystemFormatVersion, config.Version());
}

//TODO Add test cases ensuring that the values entered are correctly taken
