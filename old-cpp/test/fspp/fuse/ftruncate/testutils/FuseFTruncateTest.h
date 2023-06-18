#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_FTRUNCATE_TESTUTILS_FUSEFTRUNCATETEST_H_
#define MESSMER_FSPP_TEST_FUSE_FTRUNCATE_TESTUTILS_FUSEFTRUNCATETEST_H_

#include "../../../testutils/FuseTest.h"
#include "../../../testutils/OpenFileHandle.h"

class FuseFTruncateTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void FTruncateFile(const char *filename, fspp::num_bytes_t size);
  int FTruncateFileReturnError(const char *filename, fspp::num_bytes_t size);

private:
  cpputils::unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs, const char *filename);
};

#endif
