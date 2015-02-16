#pragma once
#ifndef TEST_FSPP_FUSE_ACCESS_TESTUTILS_FUSEACCESSTEST_H_
#define TEST_FSPP_FUSE_ACCESS_TESTUTILS_FUSEACCESSTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseAccessTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void AccessFile(const char *filename, int mode);
  int AccessFileReturnError(const char *filename, int mode);
};

#endif
