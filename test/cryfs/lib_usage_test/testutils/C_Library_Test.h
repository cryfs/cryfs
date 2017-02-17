#pragma once
#ifndef CRYFS_TEST_LIBUSAGETEST_TESTUTILS_CLIBRARYTEST_H
#define CRYFS_TEST_LIBUSAGETEST_TESTUTILS_CLIBRARYTEST_H

#include <gtest/gtest.h>
#include <cryfs/cryfs.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/tempfile/TempFile.h>

#define EXPECT_SUCCESS(command) EXPECT_EQ(cryfs_success, command)
#define EXPECT_FAIL(command) EXPECT_NE(cryfs_success, command)

class C_Library_Test : public ::testing::Test {
public:
    static constexpr uint32_t API_VERSION = 1;
    C_Library_Test() {
        EXPECT_SUCCESS(cryfs_init(API_VERSION, &api));
    }
    ~C_Library_Test() {
        cryfs_free(api);
    }

    cryfs_api_context *api;
};

#endif
