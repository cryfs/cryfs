#pragma once
#ifndef TEST_FSPP_FUSE_UTIMENS_TESTUTILS_FUSEUTIMENSTEST_H_
#define TEST_FSPP_FUSE_UTIMENS_TESTUTILS_FUSEUTIMENSTEST_H_

#include "test/testutils/FuseTest.h"

class FuseUtimensTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";
  struct timespec TIMEVALUES[2] = {makeTimespec(0,0), makeTimespec(0,0)};

  void Utimens(const char *filename, const timespec times[2]);
  int UtimensReturnError(const char *filename, const timespec times[2]);

  static struct timespec makeTimespec(time_t tv_sec, long tv_nsec);
};

MATCHER_P(TimeSpecEq, expected, "") {
  return expected[0].tv_sec == arg[0].tv_sec && expected[0].tv_nsec == arg[0].tv_nsec &&
      expected[1].tv_sec == arg[1].tv_sec && expected[1].tv_nsec == arg[1].tv_nsec;
}

#endif
