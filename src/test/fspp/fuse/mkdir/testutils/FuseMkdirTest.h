#pragma once
#ifndef TEST_FSPP_FUSE_MKDIR_TESTUTILS_FUSEMKDIRTEST_H_
#define TEST_FSPP_FUSE_MKDIR_TESTUTILS_FUSEMKDIRTEST_H_

#include "test/testutils/FuseTest.h"

class FuseMkdirTest: public FuseTest {
public:
  const char *DIRNAME = "/mydir";

  void Mkdir(const char *dirname, mode_t mode);
  int MkdirReturnError(const char *dirname, mode_t mode);

  ::testing::Action<void(const char*, mode_t)> FromNowOnReturnIsDirOnLstat();
};

#endif
