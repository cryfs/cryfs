#include "FuseRenameTest.h"
#include <cerrno>
#include <cstdio>
#include <gtest/gtest.h>


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
