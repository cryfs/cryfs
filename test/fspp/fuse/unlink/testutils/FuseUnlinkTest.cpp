#include "FuseUnlinkTest.h"
#include <boost/filesystem/path.hpp>
#include <cerrno>
#include <gtest/gtest.h>
#include <unistd.h>

using ::testing::Action;
using ::testing::Invoke;

void FuseUnlinkTest::Unlink(const char *filename) {
  const int error = UnlinkReturnError(filename);
  EXPECT_EQ(0, error);
}

int FuseUnlinkTest::UnlinkReturnError(const char *filename) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  const int retval = ::unlink(realpath.string().c_str());
  if (0 == retval) {
    return 0;
  } else {
    return errno;
  }
}

Action<void(const boost::filesystem::path&)> FuseUnlinkTest::FromNowOnReturnDoesntExistOnLstat() {
  return Invoke([this](const boost::filesystem::path& filename) {
    ReturnDoesntExistOnLstat(filename);
  });
}
