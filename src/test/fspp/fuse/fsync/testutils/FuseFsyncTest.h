#pragma once
#ifndef TEST_FSPP_FUSE_FSYNC_TESTUTILS_FUSEFSYNCTEST_H_
#define TEST_FSPP_FUSE_FSYNC_TESTUTILS_FUSEFSYNCTEST_H_

#include "test/testutils/FuseTest.h"

class FuseFsyncTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void FsyncFile(const char *filename);
  int FsyncFileReturnError(const char *filename);

private:
  int OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
