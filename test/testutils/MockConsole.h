#pragma once
#ifndef MESSMER_CRYFS_TEST_TESTUTILS_MOCKCONSOLE_H
#define MESSMER_CRYFS_TEST_TESTUTILS_MOCKCONSOLE_H

#include <messmer/cpp-utils/io/Console.h>
#include <google/gmock/gmock.h>

class MockConsole: public cpputils::Console {
public:
    MOCK_METHOD1(print, void(const std::string&));
    MOCK_METHOD2(ask, unsigned int(const std::string&, const std::vector<std::string>&));
    MOCK_METHOD1(askYesNo, bool(const std::string&));
};

#endif
