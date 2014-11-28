#include "FuseAccessTest.h"

void FuseAccessTest::AccessFile(const char *filename, int mode) {
  int retval = AccessFileAllowError(filename, mode);
  EXPECT_EQ(0, retval);
}

int FuseAccessTest::AccessFileAllowError(const char *filename, int mode) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  return ::access(realpath.c_str(), mode);
}
