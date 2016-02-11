#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_
#define MESSMER_FSPP_TEST_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseFstatTest: public FuseTest {
public:
  int CreateFile(const TempTestFS *fs, const std::string &filename);
  int CreateFileReturnError(const TempTestFS *fs, const std::string &filename);
  void OnCreateAndOpenReturnFileDescriptor(const char *filename, int descriptor);
private:
  int CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename);
};


#endif
