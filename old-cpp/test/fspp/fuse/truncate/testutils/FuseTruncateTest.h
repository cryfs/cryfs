#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_TRUNCATE_TESTUTILS_FUSETRUNCATETEST_H_
#define MESSMER_FSPP_TEST_FUSE_TRUNCATE_TESTUTILS_FUSETRUNCATETEST_H_

#include "../../../testutils/FuseTest.h"

class FuseTruncateTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void TruncateFile(const char *filename, fspp::num_bytes_t size);
  int TruncateFileReturnError(const char *filename, fspp::num_bytes_t size);
};

#endif
