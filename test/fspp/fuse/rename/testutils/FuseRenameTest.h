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

  // rename(2) cannot carry flags, so exercising the renameat2() flags needs a separate entry
  // point. Returns the errno, or 0 on success.
  int Renameat2ReturnError(const char *from, const char *to, unsigned int flags);
};

#endif
