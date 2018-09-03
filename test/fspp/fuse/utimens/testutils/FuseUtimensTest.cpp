#include "FuseUtimensTest.h"
#include <cpp-utils/system/filetime.h>

void FuseUtimensTest::Utimens(const char *filename, timespec lastAccessTime, timespec lastModificationTime) {
  int error = UtimensReturnError(filename, lastAccessTime, lastModificationTime);
  EXPECT_EQ(0, error);
}

int FuseUtimensTest::UtimensReturnError(const char *filename, timespec lastAccessTime, timespec lastModificationTime) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;

  return cpputils::set_filetime(realpath.string().c_str(), lastAccessTime, lastModificationTime);
}

struct timespec FuseUtimensTest::makeTimespec(time_t tv_sec, long tv_nsec) {
  struct timespec result{};
  result.tv_sec = tv_sec;
  result.tv_nsec = tv_nsec;
  return result;
}
