#include "FuseRmdirTest.h"

using ::testing::Action;
using ::testing::Invoke;

void FuseRmdirTest::Rmdir(const char *dirname) {
  int retval = RmdirAllowError(dirname);
  EXPECT_EQ(0, retval);
}

int FuseRmdirTest::RmdirAllowError(const char *dirname) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / dirname;
  return ::rmdir(realpath.c_str());
}

Action<void(const char*)> FuseRmdirTest::FromNowOnReturnDoesntExistOnLstat() {
  return Invoke([this](const char *dirname) {
    ReturnDoesntExistOnLstat(dirname);
  });
}
