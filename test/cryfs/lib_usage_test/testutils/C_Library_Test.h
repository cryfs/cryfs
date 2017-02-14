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
        EXPECT_SUCCESS(cryfs_load_init(API_VERSION, &context));
    }
    ~C_Library_Test() {
        cryfs_load_free(context);
    }

    cryfs_load_context *context;

    void reinit_context() {
        cryfs_load_free(context);
        EXPECT_SUCCESS(cryfs_load_init(API_VERSION, &context));
    }

    void EXPECT_LOAD_SUCCESS() {
        cryfs_mount_handle *handle = nullptr;
        EXPECT_EQ(cryfs_success, cryfs_load(context, &handle));
        EXPECT_NE(nullptr, handle);
    }

    void EXPECT_LOAD_ERROR(cryfs_status error) {
        cryfs_mount_handle *handle = nullptr;
        EXPECT_EQ(error, cryfs_load(context, &handle));
        EXPECT_EQ(nullptr, handle);
    }
};

#endif
