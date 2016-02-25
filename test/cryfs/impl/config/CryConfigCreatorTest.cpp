#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cryfs/impl/config/CryConfigCreator.h>
#include <cryfs/impl/config/CryCipher.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "../testutils/MockConsole.h"

using namespace cryfs;

using boost::optional;
using boost::none;
using cpputils::Console;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::string;
using std::vector;
using std::shared_ptr;
using std::make_shared;
using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::ValuesIn;
using ::testing::HasSubstr;
using ::testing::UnorderedElementsAreArray;
using ::testing::WithParamInterface;

class CryConfigCreatorTest: public ::testing::Test {
public:
    CryConfigCreatorTest()
            : console(make_shared<MockConsole>()),
              creator(console, cpputils::Random::PseudoRandom(), false),
              noninteractiveCreator(console, cpputils::Random::PseudoRandom(), true) {
    }
    shared_ptr<MockConsole> console;
    CryConfigCreator creator;
    CryConfigCreator noninteractiveCreator;
};

#define EXPECT_ASK_FOR_CIPHER()                                                                                        \
  EXPECT_CALL(*console, askYesNo("Use default settings?")).Times(1).WillOnce(Return(false));                           \
  EXPECT_CALL(*console, ask(HasSubstr("block cipher"), UnorderedElementsAreArray(CryCiphers::supportedCipherNames()))).Times(1)
#define EXPECT_DOES_NOT_ASK_FOR_CIPHER()                                                                               \
  EXPECT_CALL(*console, askYesNo("Use default settings?")).Times(0);                                                   \
  EXPECT_CALL(*console, ask(HasSubstr("block cipher"), _)).Times(0);

TEST_F(CryConfigCreatorTest, DoesAskForCipherIfNotSpecified) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseAnyCipher());
    CryConfig config = creator.create(none);
}

TEST_F(CryConfigCreatorTest, DoesNotAskForCipherIfSpecified) {
    EXPECT_DOES_NOT_ASK_FOR_CIPHER();
    CryConfig config = creator.create(string("aes-256-gcm"));
}

TEST_F(CryConfigCreatorTest, DoesNotAskForCipherIfNoninteractive) {
    EXPECT_DOES_NOT_ASK_FOR_CIPHER();
    CryConfig config = noninteractiveCreator.create(none);
}

TEST_F(CryConfigCreatorTest, ChoosesEmptyRootBlobId) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseAnyCipher());
    CryConfig config = creator.create(none);
    EXPECT_EQ("", config.RootBlob()); // This tells CryFS to create a new root blob
}

TEST_F(CryConfigCreatorTest, ChoosesValidEncryptionKey_448) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher("mars-448-gcm"));
    CryConfig config = creator.create(none);
    cpputils::Mars448_GCM::EncryptionKey::FromString(config.EncryptionKey()); // This crashes if invalid
}

TEST_F(CryConfigCreatorTest, ChoosesValidEncryptionKey_256) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher("aes-256-gcm"));
    CryConfig config = creator.create(none);
    cpputils::AES256_GCM::EncryptionKey::FromString(config.EncryptionKey()); // This crashes if invalid
}

TEST_F(CryConfigCreatorTest, ChoosesValidEncryptionKey_128) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher("aes-128-gcm"));
    CryConfig config = creator.create(none);
    cpputils::AES128_GCM::EncryptionKey::FromString(config.EncryptionKey()); // This crashes if invalid
}
