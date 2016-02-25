#pragma once
#ifndef MESSMER_CRYFS_TEST_TESTUTILS_MOCKCONSOLE_H
#define MESSMER_CRYFS_TEST_TESTUTILS_MOCKCONSOLE_H

#include <cpp-utils/io/Console.h>
#include <gmock/gmock.h>

class MockConsole: public cpputils::Console {
public:
    MOCK_METHOD1(print, void(const std::string&));
    MOCK_METHOD2(ask, unsigned int(const std::string&, const std::vector<std::string>&));
    MOCK_METHOD1(askYesNo, bool(const std::string&));
};

ACTION_P(ChooseCipher, cipherName) {
    return std::find(arg1.begin(), arg1.end(), cipherName) - arg1.begin();
}

#define ChooseAnyCipher() ChooseCipher("aes-256-gcm")

class TestWithMockConsole {
public:
    // Return a console that chooses a valid cryfs setting
    static std::shared_ptr<MockConsole> mockConsole() {
        auto console = std::make_shared<MockConsole>();
        EXPECT_CALL(*console, ask(::testing::_, ::testing::_)).WillRepeatedly(ChooseCipher("aes-256-gcm"));
        EXPECT_CALL(*console, askYesNo(::testing::_)).WillRepeatedly(::testing::Return(true));
        return console;
    }
};

#endif
