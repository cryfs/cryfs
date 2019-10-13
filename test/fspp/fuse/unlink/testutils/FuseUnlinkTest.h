#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_UNLINK_TESTUTILS_FUSEUNLINKTEST_H_
#define MESSMER_FSPP_TEST_FUSE_UNLINK_TESTUTILS_FUSEUNLINKTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseUnlinkTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void Unlink(const char *filename);
  int UnlinkReturnError(const char *filename);

  ::testing::Action<void(const boost::filesystem::path&)> FromNowOnReturnDoesntExistOnLstat();
};

#endif
