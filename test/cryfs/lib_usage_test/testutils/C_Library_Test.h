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
    C_Library_Test() {
        EXPECT_SUCCESS(cryfs_load_init(&context));
    }
    ~C_Library_Test() {
        cryfs_load_free(context);
    }

    cryfs_load_context *context;
};

#endif
