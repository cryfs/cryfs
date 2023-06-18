#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_RENAME_TESTUTILS_FUSERENAMETEST_H_
#define MESSMER_FSPP_TEST_FUSE_RENAME_TESTUTILS_FUSERENAMETEST_H_

#include "../../../testutils/FuseTest.h"

class FuseRenameTest: public FuseTest {
public:
  const char *FILENAME1 = "/myfile1";
  const char *FILENAME2 = "/myfile2";

  void Rename(const char *from, const char *to);
  int RenameReturnError(const char *from, const char *to);
};

#endif
