#include <test/fspp/fuse/mkdir/testutils/FuseMkdirTest.h>

using ::testing::Action;
using ::testing::Invoke;

void FuseMkdirTest::Mkdir(const char *dirname, mode_t mode) {
  int retval = MkdirAllowError(dirname, mode);
  EXPECT_EQ(0, retval);
}

int FuseMkdirTest::MkdirAllowError(const char *dirname, mode_t mode) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / dirname;
  return ::mkdir(realpath.c_str(), mode);
}

Action<void(const char*, mode_t)> FuseMkdirTest::FromNowOnReturnIsDirOnLstat() {
  return Invoke([this](const char *dirname, mode_t) {
    ReturnIsDirOnLstat(dirname);
  });
}
