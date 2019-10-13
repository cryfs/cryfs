#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_MKDIR_TESTUTILS_FUSEMKDIRTEST_H_
#define MESSMER_FSPP_TEST_FUSE_MKDIR_TESTUTILS_FUSEMKDIRTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseMkdirTest: public FuseTest {
public:
  const char *DIRNAME = "/mydir";

  void Mkdir(const char *dirname, mode_t mode);
  int MkdirReturnError(const char *dirname, mode_t mode);

  ::testing::Action<void(const boost::filesystem::path&, mode_t, uid_t, gid_t)> FromNowOnReturnIsDirOnLstat();
};

#endif
