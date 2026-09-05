#include "FuseUtimensTest.h"
#include <cpp-utils/system/filetime.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <cerrno>

void FuseUtimensTest::Utimens(const char *filename, timespec lastAccessTime, timespec lastModificationTime) {
  const int error = UtimensReturnError(filename, lastAccessTime, lastModificationTime);
  EXPECT_EQ(0, error);
}

int FuseUtimensTest::UtimensReturnError(const char *filename, timespec lastAccessTime, timespec lastModificationTime) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;

  return cpputils::set_filetime(realpath.string().c_str(), lastAccessTime, lastModificationTime);
}

void FuseUtimensTest::Utimensat(const char *filename, const struct timespec *times) {
  const int error = UtimensatReturnError(filename, times);
  EXPECT_EQ(0, error);
}

int FuseUtimensTest::UtimensatReturnError(const char *filename, const struct timespec *times) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;

  const int retval = ::utimensat(AT_FDCWD, realpath.string().c_str(), times, 0);
  return (0 == retval) ? 0 : errno;
}

struct timespec FuseUtimensTest::makeTimespec(time_t tv_sec, long tv_nsec) {
  struct timespec result{};
  result.tv_sec = tv_sec;
  result.tv_nsec = tv_nsec;
  return result;
}
