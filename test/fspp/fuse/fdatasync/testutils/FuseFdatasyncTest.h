#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_FDATASYNC_TESTUTILS_FUSEFDATASYNCTEST_H_
#define MESSMER_FSPP_TEST_FUSE_FDATASYNC_TESTUTILS_FUSEFDATASYNCTEST_H_

#include "../../../testutils/FuseTest.h"
#include "../../../testutils/OpenFileHandle.h"

class FuseFdatasyncTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void FdatasyncFile(const char *filename);
  int FdatasyncFileReturnError(const char *filename);

private:
  cpputils::unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
