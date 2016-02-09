#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_UTIMENS_TESTUTILS_FUSEUTIMENSTEST_H_
#define MESSMER_FSPP_TEST_FUSE_UTIMENS_TESTUTILS_FUSEUTIMENSTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseUtimensTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";
  timespec TIMEVALUE = makeTimespec(0,0);

  void Utimens(const char *filename, timespec lastAccessTime, timespec lastModificationTime);
  int UtimensReturnError(const char *filename, timespec lastAccessTime, timespec lastModificationTime);

  static struct timespec makeTimespec(time_t tv_sec, long tv_nsec);
};

MATCHER_P(TimeSpecEq, expected, "") {
  return expected.tv_sec == arg.tv_sec && expected.tv_nsec == arg.tv_nsec;
}

#endif
