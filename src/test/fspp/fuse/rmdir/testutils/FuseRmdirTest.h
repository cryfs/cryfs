#pragma once
#ifndef TEST_FSPP_FUSE_RMDIR_TESTUTILS_FUSERMDIRTEST_H_
#define TEST_FSPP_FUSE_RMDIR_TESTUTILS_FUSERMDIRTEST_H_

#include "test/testutils/FuseTest.h"

class FuseRmdirTest: public FuseTest {
public:
  const char *DIRNAME = "/mydir";

  void Rmdir(const char *dirname);
  int RmdirAllowError(const char *dirname);

  ::testing::Action<void(const char*)> FromNowOnReturnDoesntExistOnLstat();
};

#endif
