#ifndef CRYFS_TEST_PROGRAMOPTIONS_PROGRAMOPTIONSTEST_H
#define CRYFS_TEST_PROGRAMOPTIONS_PROGRAMOPTIONSTEST_H

#include <google/gtest/gtest.h>

class ProgramOptionsTestBase: public ::testing::Test {
public:
    std::vector<char*> options(std::initializer_list<const char*> options) {
        std::vector<char*> result;
        for (auto option : options) {
            result.push_back(const_cast<char*>(option));
        }
        return result;
    }

    void EXPECT_VECTOR_EQ(std::initializer_list<const char*> expected, const std::vector<char*> &actual) {
        std::vector<const char*> expectedVec(expected);
        EXPECT_EQ(expectedVec.size(), actual.size());
        for(unsigned int i = 0; i < expectedVec.size(); ++i) {
            EXPECT_EQ(std::string(expectedVec[i]), std::string(actual[i]));
        }
    }
};

#endif
