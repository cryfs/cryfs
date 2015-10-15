#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_WRITE_TESTUTILS_FUSEWRITETEST_H_
#define MESSMER_FSPP_TEST_FUSE_WRITE_TESTUTILS_FUSEWRITETEST_H_

#include "../../../testutils/FuseTest.h"

class FuseWriteTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  struct WriteError {
    int error;
    size_t written_bytes;
  };

  void WriteFile(const char *filename, const void *buf, size_t count, off_t offset);
  WriteError WriteFileReturnError(const char *filename, const void *buf, size_t count, off_t offset);

private:
  int OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
