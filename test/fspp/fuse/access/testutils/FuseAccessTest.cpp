#include "FuseAccessTest.h"

void FuseAccessTest::AccessFile(const char *filename, int mode) {
  int error = AccessFileReturnError(filename, mode);
  EXPECT_EQ(0, error);
}

int FuseAccessTest::AccessFileReturnError(const char *filename, int mode) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  int retval = ::access(realpath.string().c_str(), mode);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}
