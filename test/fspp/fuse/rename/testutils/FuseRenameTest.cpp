#include "FuseRenameTest.h"


void FuseRenameTest::Rename(const char *from, const char *to) {
  int error = RenameReturnError(from, to);
  EXPECT_EQ(0, error);
}

int FuseRenameTest::RenameReturnError(const char *from, const char *to) {
  auto fs = TestFS();

  auto realfrom = fs->mountDir() / from;
  auto realto = fs->mountDir() / to;
  int retval = ::rename(realfrom.c_str(), realto.c_str());
  if (0 == retval) {
    return 0;
  } else {
    return errno;
  }
}
