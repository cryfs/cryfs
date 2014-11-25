#pragma once
#ifndef TEST_FSPP_FUSE_TRUNCATE_TESTUTILS_FUSETRUNCATETEST_H_
#define TEST_FSPP_FUSE_TRUNCATE_TESTUTILS_FUSETRUNCATETEST_H_

#include "test/testutils/FuseTest.h"

class FuseTruncateTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void TruncateFile(const char *filename, off_t size);
  int TruncateFileAllowError(const char *filename, off_t size);
};

#endif
