#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_FSYNC_TESTUTILS_FUSEFSYNCTEST_H_
#define MESSMER_FSPP_TEST_FUSE_FSYNC_TESTUTILS_FUSEFSYNCTEST_H_

#include "../../../testutils/FuseTest.h"
#include "../../../testutils/OpenFileHandle.h"

class FuseFsyncTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void FsyncFile(const char *filename);
  int FsyncFileReturnError(const char *filename);

private:
  cpputils::unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
