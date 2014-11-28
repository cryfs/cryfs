#include "FuseUtimensTest.h"

#include <utime.h>
#include <sys/time.h>

void FuseUtimensTest::Utimens(const char *filename, const timespec times[2]) {
  int retval = UtimensAllowError(filename, times);
  EXPECT_EQ(0, retval);
}

int FuseUtimensTest::UtimensAllowError(const char *filename, const timespec times[2]) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;

  struct timeval casted_times[2];
  TIMESPEC_TO_TIMEVAL(&casted_times[0], &times[0]);
  TIMESPEC_TO_TIMEVAL(&casted_times[1], &times[1]);
  return ::utimes(realpath.c_str(), casted_times);
}

struct timespec FuseUtimensTest::makeTimespec(time_t tv_sec, long tv_nsec) {
  struct timespec result;
  result.tv_sec = tv_sec;
  result.tv_nsec = tv_nsec;
  return result;
}
