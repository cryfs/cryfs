#include "FuseRenameTest.h"

#include <fcntl.h>
#include <cerrno>
#if defined(__linux__)
#include <sys/syscall.h>
#include <unistd.h>
#endif


void FuseRenameTest::Rename(const char *from, const char *to) {
  const int error = RenameReturnError(from, to);
  EXPECT_EQ(0, error);
}

int FuseRenameTest::RenameReturnError(const char *from, const char *to) {
  auto fs = TestFS();

  auto realfrom = fs->mountDir() / from;
  auto realto = fs->mountDir() / to;
  const int retval = ::rename(realfrom.string().c_str(), realto.string().c_str());
  if (0 == retval) {
    return 0;
  } else {
    return errno;
  }
}

int FuseRenameTest::Renameat2ReturnError(const char *from, const char *to, unsigned int flags) {
#if defined(__linux__)
  auto fs = TestFS();

  auto realfrom = fs->mountDir() / from;
  auto realto = fs->mountDir() / to;
  // Call the syscall directly: glibc only grew a renameat2() wrapper in 2.28.
  const int retval = ::syscall(SYS_renameat2, AT_FDCWD, realfrom.string().c_str(), AT_FDCWD, realto.string().c_str(), flags);
  if (0 == retval) {
    return 0;
  } else {
    return errno;
  }
#else
  UNUSED(from); UNUSED(to); UNUSED(flags);
  return ENOSYS;
#endif
}
