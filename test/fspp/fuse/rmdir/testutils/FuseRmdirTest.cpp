#include "FuseRmdirTest.h"

using ::testing::Action;
using ::testing::Invoke;

void FuseRmdirTest::Rmdir(const char *dirname) {
  int error = RmdirReturnError(dirname);
  EXPECT_EQ(0, error);
}

int FuseRmdirTest::RmdirReturnError(const char *dirname) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / dirname;
  int retval = ::rmdir(realpath.string().c_str());
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

Action<void(const char*)> FuseRmdirTest::FromNowOnReturnDoesntExistOnLstat() {
  return Invoke([this](const char *dirname) {
    ReturnDoesntExistOnLstat(dirname);
  });
}
