#pragma once
#ifndef TEST_FSPP_FUSE_FDATASYNC_TESTUTILS_FUSEFDATASYNCTEST_H_
#define TEST_FSPP_FUSE_FDATASYNC_TESTUTILS_FUSEFDATASYNCTEST_H_

#include "test/testutils/FuseTest.h"

class FuseFdatasyncTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void FdatasyncFile(const char *filename);
  int FdatasyncFileReturnError(const char *filename);

private:
  int OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
