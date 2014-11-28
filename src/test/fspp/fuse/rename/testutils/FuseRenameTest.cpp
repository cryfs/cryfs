#include "FuseRenameTest.h"

using ::testing::Action;
using ::testing::Invoke;

void FuseRenameTest::Rename(const char *from, const char *to) {
  int retval = RenameAllowError(from, to);
  EXPECT_EQ(0, retval);
}

int FuseRenameTest::RenameAllowError(const char *from, const char *to) {
  auto fs = TestFS();

  auto realfrom = fs->mountDir() / from;
  auto realto = fs->mountDir() / to;
  return ::rename(realfrom.c_str(), realto.c_str());
}
