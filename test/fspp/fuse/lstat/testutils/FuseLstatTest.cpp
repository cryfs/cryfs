#include "FuseLstatTest.h"

using std::function;
using ::testing::Eq;
using ::testing::Invoke;

void FuseLstatTest::LstatPath(const std::string &path) {
  fspp::fuse::STAT dummy{};
  LstatPath(path, &dummy);
}

int FuseLstatTest::LstatPathReturnError(const std::string &path) {
  fspp::fuse::STAT dummy{};
  return LstatPathReturnError(path, &dummy);
}

void FuseLstatTest::LstatPath(const std::string &path, fspp::fuse::STAT *result) {
  int error = LstatPathReturnError(path, result);
  EXPECT_EQ(0, error) << "lstat syscall failed. errno: " << error;
}

int FuseLstatTest::LstatPathReturnError(const std::string &path, fspp::fuse::STAT *result) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / path;
  int retval = ::lstat(realpath.string().c_str(), result);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

fspp::fuse::STAT FuseLstatTest::CallFileLstatWithImpl(function<void(fspp::fuse::STAT*)> implementation) {
  return CallLstatWithModeAndImpl(S_IFREG, implementation);
}

fspp::fuse::STAT FuseLstatTest::CallDirLstatWithImpl(function<void(fspp::fuse::STAT*)> implementation) {
  return CallLstatWithModeAndImpl(S_IFDIR, implementation);
}

fspp::fuse::STAT FuseLstatTest::CallLstatWithImpl(function<void(fspp::fuse::STAT*)> implementation) {
  EXPECT_CALL(*fsimpl, lstat(Eq(FILENAME), testing::_)).WillRepeatedly(Invoke([implementation](const boost::filesystem::path&, fspp::fuse::STAT *stat) {
    implementation(stat);
  }));

  fspp::fuse::STAT result{};
  LstatPath(FILENAME, &result);

  return result;
}

fspp::fuse::STAT FuseLstatTest::CallLstatWithModeAndImpl(mode_t mode, function<void(fspp::fuse::STAT*)> implementation) {
  return CallLstatWithImpl([mode, implementation] (fspp::fuse::STAT *stat) {
    stat->st_mode = mode;
    implementation(stat);
  });
}
