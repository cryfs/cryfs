#pragma once
#ifndef CRYFS_TEST_LIBUSAGETEST_TESTUTILS_LOADTEST_H
#define CRYFS_TEST_LIBUSAGETEST_TESTUTILS_LOADTEST_H

#include <gtest/gtest.h>
#include "C_Library_Test.h"

class Load_Test : public C_Library_Test {
public:
  Load_Test() {
    EXPECT_SUCCESS(cryfs_load_init(api, &context));
  }
  ~Load_Test() {
    EXPECT_SUCCESS(cryfs_load_free(context));
  }

  cryfs_load_context *context;

  void reinit_context() {
    EXPECT_SUCCESS(cryfs_load_free(context));
    EXPECT_SUCCESS(cryfs_load_init(api, &context));
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
