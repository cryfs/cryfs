#include "../../impl/testutils/MockConsole.h"
#include <boost/none.hpp>
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <cryfs/impl/config/CryCipher.h>
#include <cryfs/impl/config/CryConfigConsole.h>
#include <cstdint>
#include <gmock/gmock.h>
#include <gtest/gtest.h>
#include <string>

using namespace cryfs;

using boost::optional;
using boost::none;
using cpputils::NoninteractiveConsole;
using std::string;
using std::shared_ptr;
using std::make_shared;
using ::testing::NiceMock;
using ::testing::Return;
using ::testing::ValuesIn;
using ::testing::HasSubstr;
using ::testing::UnorderedElementsAreArray;
using ::testing::WithParamInterface;

class CryConfigConsoleTest: public ::testing::Test {
public:
    CryConfigConsoleTest()
            : console(make_shared<NiceMock<MockConsole>>()),
              cryconsole(console),
              noninteractiveCryconsole(make_shared<NoninteractiveConsole>(console)) {
    }
    shared_ptr<NiceMock<MockConsole>> console;
    CryConfigConsole cryconsole;
    CryConfigConsole noninteractiveCryconsole;
};

class CryConfigConsoleTest_Cipher: public CryConfigConsoleTest {};

#define EXPECT_ASK_FOR_CIPHER()                                                                                        \
  EXPECT_CALL(*console, askYesNo("Use default settings?", testing::_)).Times(1).WillOnce(Return(false));                        \
  EXPECT_CALL(*console, ask(HasSubstr("block cipher"), UnorderedElementsAreArray(CryCiphers::supportedCipherNames()))).Times(1)

#define EXPECT_ASK_FOR_BLOCKSIZE()                                                                                     \
  EXPECT_CALL(*console, askYesNo("Use default settings?", testing::_)).Times(1).WillOnce(Return(false));                        \
  EXPECT_CALL(*console, ask(HasSubstr("block size"), testing::_)).Times(1)

#define EXPECT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION()                                                              \
  EXPECT_CALL(*console, askYesNo("Use default settings?", testing::_)).Times(1).WillOnce(Return(false));                        \
  EXPECT_CALL(*console, askYesNo(HasSubstr("missing block"), testing::_)).Times(1)

TEST_F(CryConfigConsoleTest_Cipher, AsksForCipher) {
    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseAnyCipher());
    cryconsole.askCipher();
}

TEST_F(CryConfigConsoleTest_Cipher, ChooseDefaultCipher) {
    EXPECT_CALL(*console, askYesNo("Use default settings?", testing::_)).Times(1).WillOnce(Return(true));
    EXPECT_CALL(*console, ask(HasSubstr("block cipher"), testing::_)).Times(0);
    const string cipher = cryconsole.askCipher();
    EXPECT_EQ(CryConfigConsole::DEFAULT_CIPHER, cipher);
}

TEST_F(CryConfigConsoleTest_Cipher, ChooseDefaultCipherWhenNoninteractiveEnvironment) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("default"), testing::_)).Times(0);
    EXPECT_CALL(*console, ask(HasSubstr("block cipher"), testing::_)).Times(0);
    const string cipher = noninteractiveCryconsole.askCipher();
    EXPECT_EQ(CryConfigConsole::DEFAULT_CIPHER, cipher);
}

TEST_F(CryConfigConsoleTest_Cipher, AsksForBlocksize) {
    EXPECT_ASK_FOR_BLOCKSIZE().WillOnce(Return(0));
    cryconsole.askBlocksizeBytes();
}

TEST_F(CryConfigConsoleTest_Cipher, AsksForMissingBlockIsIntegrityViolation) {
    EXPECT_ASK_FOR_MISSINGBLOCKISINTEGRITYVIOLATION().WillOnce(Return(true));
    cryconsole.askMissingBlockIsIntegrityViolation();
}

TEST_F(CryConfigConsoleTest_Cipher, ChooseDefaultBlocksizeWhenNoninteractiveEnvironment) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("default"), testing::_)).Times(0);
    EXPECT_CALL(*console, ask(HasSubstr("block size"), testing::_)).Times(0);
    const uint32_t blocksize = noninteractiveCryconsole.askBlocksizeBytes();
    EXPECT_EQ(CryConfigConsole::DEFAULT_BLOCKSIZE_BYTES, blocksize);
}

class CryConfigConsoleTest_Cipher_Choose: public CryConfigConsoleTest_Cipher, public ::testing::WithParamInterface<string> {
public:
    string cipherName = GetParam();
    optional<string> cipherWarning = CryCiphers::find(cipherName).warning();

    void EXPECT_DONT_SHOW_WARNING() {
        EXPECT_CALL(*console, askYesNo(testing::_, testing::_)).Times(0);
    }

    void EXPECT_SHOW_WARNING(const string &warning) {
        EXPECT_CALL(*console, askYesNo(HasSubstr(warning), testing::_)).WillOnce(Return(true));
    }
};

TEST_P(CryConfigConsoleTest_Cipher_Choose, ChoosesCipherCorrectly) {
    if (cipherWarning == none) {
        EXPECT_DONT_SHOW_WARNING();
    } else {
        EXPECT_SHOW_WARNING(*cipherWarning);
    }

    EXPECT_ASK_FOR_CIPHER().WillOnce(ChooseCipher(cipherName));

    const string chosenCipher = cryconsole.askCipher();
    EXPECT_EQ(cipherName, chosenCipher);
}

INSTANTIATE_TEST_SUITE_P(CryConfigConsoleTest_Cipher_Choose, CryConfigConsoleTest_Cipher_Choose, ValuesIn(CryCiphers::supportedCipherNames()));
