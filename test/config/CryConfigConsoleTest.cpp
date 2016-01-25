#include <google/gtest/gtest.h>
#include <google/gmock/gmock.h>
#include "../../src/config/CryConfigConsole.h"
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

class CryConfigConsoleTest: public ::testing::Test {
public:
    CryConfigConsoleTest()
            : console(make_shared<MockConsole>()),
              cryconsole(console) {
    }
    shared_ptr<MockConsole> console;
    CryConfigConsole cryconsole;
};

class CryConfigConsoleTest_Cipher: public CryConfigConsoleTest {};

#define EXPECT_ASK_FOR_CIPHER() EXPECT_CALL(*console, ask(HasSubstr("block cipher"), UnorderedElementsAreArray(CryCiphers::supportedCipherNames())))

TEST_F(CryConfigConsoleTest_Cipher, AsksForCipher) {
    EXPECT_ASK_FOR_CIPHER().Times(1).WillOnce(ChooseAnyCipher());
    cryconsole.askCipher();
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
