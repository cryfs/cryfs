#include <google/gtest/gtest.h>
#include <google/gmock/gmock.h>
#include "../../src/config/CryConfigCreator.h"
#include "../../src/config/CryCipher.h"
#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>
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
              creator(console, cpputils::Random::PseudoRandom()) {
    }
    shared_ptr<MockConsole> console;
    CryConfigCreator creator;
};

#define EXPECT_ASK_FOR_CIPHER() EXPECT_CALL(*console, ask(HasSubstr("block cipher"), UnorderedElementsAreArray(CryCiphers::supportedCipherNames())))

TEST_F(CryConfigCreatorTest, DoesAskForCipherIfNotSpecified) {
    EXPECT_ASK_FOR_CIPHER().Times(1).WillOnce(ChooseAnyCipher());
    CryConfig config = creator.create(none);
}

TEST_F(CryConfigCreatorTest, DoesNotAskForCipherIfSpecified) {
    EXPECT_ASK_FOR_CIPHER().Times(0);
    CryConfig config = creator.create(string("aes-256-gcm"));
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

class CryConfigCreatorTest_ChooseCipher: public CryConfigCreatorTest, public ::testing::WithParamInterface<string> {
public:
    string cipherName = GetParam();
    optional<string> cipherWarning = CryCiphers::find(cipherName).warning();

    void EXPECT_DONT_SHOW_WARNING() {
        EXPECT_CALL(*console, askYesNo(_)).Times(0);
    }

    void EXPECT_SHOW_WARNING(const string &warning) {
        EXPECT_CALL(*console, askYesNo(HasSubstr(warning))).WillOnce(Return(true));
    }
};

TEST_P(CryConfigCreatorTest_ChooseCipher, ChoosesCipherCorrectly) {
    if (cipherWarning == none) {
        EXPECT_DONT_SHOW_WARNING();
    } else {
        EXPECT_SHOW_WARNING(*cipherWarning);
    }

    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher(cipherName));

    CryConfig config = creator.create(none);
    EXPECT_EQ(cipherName, config.Cipher());
}

INSTANTIATE_TEST_CASE_P(CryConfigCreatorTest_ChooseCipher, CryConfigCreatorTest_ChooseCipher, ValuesIn(CryCiphers::supportedCipherNames()));
