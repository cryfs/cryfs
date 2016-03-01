#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cryfs/config/CryConfigConsole.h>
#include <cryfs/config/CryCipher.h>
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

class CryConfigConsoleTest: public ::testing::Test {
public:
    CryConfigConsoleTest()
            : console(make_shared<MockConsole>()),
              cryconsole(console, false),
              noninteractiveCryconsole(console, true) {
    }
    shared_ptr<MockConsole> console;
    CryConfigConsole cryconsole;
    CryConfigConsole noninteractiveCryconsole;
};

class CryConfigConsoleTest_Cipher: public CryConfigConsoleTest {};

#define EXPECT_ASK_FOR_CIPHER()                                                                                        \
  EXPECT_CALL(*console, askYesNo("Use default settings?")).Times(1).WillOnce(Return(false));                           \
  EXPECT_CALL(*console, ask(HasSubstr("block cipher"), UnorderedElementsAreArray(CryCiphers::supportedCipherNames()))).Times(1)

#define EXPECT_ASK_FOR_BLOCKSIZE()                                                                                     \
  EXPECT_CALL(*console, askYesNo("Use default settings?")).Times(1).WillOnce(Return(false));                           \
  EXPECT_CALL(*console, ask(HasSubstr("block size"), _)).Times(1)

TEST_F(CryConfigConsoleTest_Cipher, AsksForCipher) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseAnyCipher());
    cryconsole.askCipher();
}

TEST_F(CryConfigConsoleTest_Cipher, ChooseDefaultCipher) {
    EXPECT_CALL(*console, askYesNo("Use default settings?")).Times(1).WillOnce(Return(true));
    EXPECT_CALL(*console, ask(HasSubstr("block cipher"), _)).Times(0);
    string cipher = cryconsole.askCipher();
    EXPECT_EQ(CryConfigConsole::DEFAULT_CIPHER, cipher);
}

TEST_F(CryConfigConsoleTest_Cipher, ChooseDefaultCipherWhenNoninteractiveEnvironment) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("default"))).Times(0);
    EXPECT_CALL(*console, ask(HasSubstr("block cipher"), _)).Times(0);
    string cipher = noninteractiveCryconsole.askCipher();
    EXPECT_EQ(CryConfigConsole::DEFAULT_CIPHER, cipher);
}

TEST_F(CryConfigConsoleTest_Cipher, AsksForBlocksize) {
    EXPECT_ASK_FOR_BLOCKSIZE().WillOnce(Return(0));
    cryconsole.askBlocksizeBytes();
}

TEST_F(CryConfigConsoleTest_Cipher, ChooseDefaultBlocksizeWhenNoninteractiveEnvironment) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("default"))).Times(0);
    EXPECT_CALL(*console, ask(HasSubstr("block size"), _)).Times(0);
    uint32_t blocksize = noninteractiveCryconsole.askBlocksizeBytes();
    EXPECT_EQ(CryConfigConsole::DEFAULT_BLOCKSIZE_BYTES, blocksize);
}

class CryConfigConsoleTest_Cipher_Choose: public CryConfigConsoleTest_Cipher, public ::testing::WithParamInterface<string> {
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

TEST_P(CryConfigConsoleTest_Cipher_Choose, ChoosesCipherCorrectly) {
    if (cipherWarning == none) {
        EXPECT_DONT_SHOW_WARNING();
    } else {
        EXPECT_SHOW_WARNING(*cipherWarning);
    }

    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher(cipherName));

    string chosenCipher = cryconsole.askCipher();
    EXPECT_EQ(cipherName, chosenCipher);
}

INSTANTIATE_TEST_CASE_P(CryConfigConsoleTest_Cipher_Choose, CryConfigConsoleTest_Cipher_Choose, ValuesIn(CryCiphers::supportedCipherNames()));
