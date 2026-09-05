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

  // set_filetime() goes through utimes(2), which can only express two concrete times.
  // To exercise UTIME_NOW/UTIME_OMIT we have to call utimensat(2) ourselves.
  // Passing nullptr for times is what `touch` does: it means "both timestamps to now".
  void Utimensat(const char *filename, const struct timespec *times);
  int UtimensatReturnError(const char *filename, const struct timespec *times);

  static struct timespec makeTimespec(time_t tv_sec, long tv_nsec);
};

//NOLINTNEXTLINE(cppcoreguidelines-avoid-const-or-ref-data-members)
MATCHER_P(TimeSpecEq, expected, "") {
  return expected.tv_sec == arg.tv_sec && expected.tv_nsec == arg.tv_nsec;
}

#endif
