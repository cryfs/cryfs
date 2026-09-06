#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_
#define MESSMER_FSPP_TEST_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_

#include "../../../testutils/FuseTest.h"
#include "../../../testutils/OpenFileHandle.h"

class FuseFstatTest: public FuseTest {
public:
  cpputils::unique_ref<OpenFileHandle> CreateFile(const TempTestFS *fs, const std::string &filename);
  int CreateFileReturnError(const TempTestFS *fs, const std::string &filename);
  void OnCreateAndOpenReturnFileDescriptor(const char *filename, int descriptor);
private:
  cpputils::unique_ref<OpenFileHandle> CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename);
};


#endif
