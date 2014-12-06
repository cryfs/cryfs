#pragma once
#ifndef TEST_FSPP_FUSE_FTRUNCATE_TESTUTILS_FUSEFTRUNCATETEST_H_
#define TEST_FSPP_FUSE_FTRUNCATE_TESTUTILS_FUSEFTRUNCATETEST_H_

#include "test/testutils/FuseTest.h"

class FuseFTruncateTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void FTruncateFile(const char *filename, off_t size);
  int FTruncateFileReturnError(const char *filename, off_t size);

private:
  int OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
