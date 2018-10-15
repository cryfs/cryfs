#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_WRITE_TESTUTILS_FUSEWRITETEST_H_
#define MESSMER_FSPP_TEST_FUSE_WRITE_TESTUTILS_FUSEWRITETEST_H_

#include "../../../testutils/FuseTest.h"
#include "../../../testutils/OpenFileHandle.h"

class FuseWriteTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  struct WriteError {
    int error{};
    fspp::num_bytes_t written_bytes;
  };

  void WriteFile(const char *filename, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset);
  WriteError WriteFileReturnError(const char *filename, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset);

private:
  cpputils::unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
