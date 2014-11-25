#include "FuseTruncateTest.h"

void FuseTruncateTest::TruncateFile(const char *filename, off_t size) {
  int retval = TruncateFileAllowError(filename, size);
  EXPECT_EQ(0, retval);
}

int FuseTruncateTest::TruncateFileAllowError(const char *filename, off_t size) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  return ::truncate(realpath.c_str(), size);
}
