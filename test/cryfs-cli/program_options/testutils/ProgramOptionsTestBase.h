#pragma once
#ifndef MESSMER_CRYFS_TEST_PROGRAMOPTIONS_PROGRAMOPTIONSTEST_H
#define MESSMER_CRYFS_TEST_PROGRAMOPTIONS_PROGRAMOPTIONSTEST_H

#include <gtest/gtest.h>

class ProgramOptionsTestBase: public ::testing::Test {
public:

    void EXPECT_VECTOR_EQ(std::initializer_list<std::string> expected, const std::vector<std::string> &actual) {
        std::vector<std::string> expectedVec(expected);
        ASSERT_EQ(expectedVec.size(), actual.size());
        for(size_t i = 0; i < expectedVec.size(); ++i) {
            EXPECT_EQ(expectedVec[i], actual[i]);
        }
    }
};

#endif
