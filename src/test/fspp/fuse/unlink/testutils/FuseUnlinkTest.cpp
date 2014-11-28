#include <test/fspp/fuse/unlink/testutils/FuseUnlinkTest.h>

using ::testing::Action;
using ::testing::Invoke;

void FuseUnlinkTest::Unlink(const char *filename) {
  int retval = UnlinkAllowError(filename);
  EXPECT_EQ(0, retval);
}

int FuseUnlinkTest::UnlinkAllowError(const char *filename) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  return ::unlink(realpath.c_str());
}

Action<void(const char*)> FuseUnlinkTest::FromNowOnReturnDoesntExistOnLstat() {
  return Invoke([this](const char *filename) {
    ReturnDoesntExistOnLstat(filename);
  });
}
