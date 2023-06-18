#include "FuseMkdirTest.h"

using ::testing::Action;
using ::testing::Invoke;

void FuseMkdirTest::Mkdir(const char *dirname, mode_t mode) {
  int error = MkdirReturnError(dirname, mode);
  EXPECT_EQ(0, error);
}

int FuseMkdirTest::MkdirReturnError(const char *dirname, mode_t mode) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / dirname;
  int retval = ::mkdir(realpath.string().c_str(), mode);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

Action<void(const boost::filesystem::path&, mode_t, uid_t, gid_t)> FuseMkdirTest::FromNowOnReturnIsDirOnLstat() {
  return Invoke([this](const boost::filesystem::path& dirname, mode_t, uid_t, gid_t) {
    ReturnIsDirOnLstat(dirname);
  });
}
