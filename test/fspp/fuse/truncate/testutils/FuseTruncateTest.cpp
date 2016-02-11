#include "FuseTruncateTest.h"

void FuseTruncateTest::TruncateFile(const char *filename, off_t size) {
  int error = TruncateFileReturnError(filename, size);
  EXPECT_EQ(0, error);
}

int FuseTruncateTest::TruncateFileReturnError(const char *filename, off_t size) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  int retval = ::truncate(realpath.c_str(), size);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}
