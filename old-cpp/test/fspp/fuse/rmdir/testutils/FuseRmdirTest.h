#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_RMDIR_TESTUTILS_FUSERMDIRTEST_H_
#define MESSMER_FSPP_TEST_FUSE_RMDIR_TESTUTILS_FUSERMDIRTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseRmdirTest: public FuseTest {
public:
  const char *DIRNAME = "/mydir";

  void Rmdir(const char *dirname);
  int RmdirReturnError(const char *dirname);

  ::testing::Action<void(const boost::filesystem::path&)> FromNowOnReturnDoesntExistOnLstat();
};

#endif
