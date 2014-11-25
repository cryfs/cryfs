#pragma once
#ifndef TEST_FSPP_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_
#define TEST_FSPP_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_

#include "test/testutils/FuseTest.h"

class FuseFstatTest: public FuseTest {
public:
  int CreateFile(const TempTestFS *fs, const std::string &filename);
  int CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename);
  void OnCreateAndOpenReturnFileDescriptor(const char *filename, int descriptor);
};


#endif
