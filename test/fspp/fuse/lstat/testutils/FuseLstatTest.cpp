#include "FuseLstatTest.h"

using std::function;
using ::testing::StrEq;
using ::testing::_;
using ::testing::Invoke;

void FuseLstatTest::LstatPath(const std::string &path) {
  struct stat dummy{};
  LstatPath(path, &dummy);
}

int FuseLstatTest::LstatPathReturnError(const std::string &path) {
  struct stat dummy{};
  return LstatPathReturnError(path, &dummy);
}

void FuseLstatTest::LstatPath(const std::string &path, struct stat *result) {
  int error = LstatPathReturnError(path, result);
  EXPECT_EQ(0, error) << "lstat syscall failed. errno: " << error;
}

int FuseLstatTest::LstatPathReturnError(const std::string &path, struct stat *result) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / path;
#if defined(_MSC_VER)
  // Windows doesn't know lstat
  int retval = ::stat(realpath.string().c_str(), result);
#else
  int retval = ::lstat(realpath.string().c_str(), result);
#endif
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

struct stat FuseLstatTest::CallFileLstatWithImpl(function<void(struct FUSE_STAT*)> implementation) {
  return CallLstatWithModeAndImpl(S_IFREG, implementation);
}

struct stat FuseLstatTest::CallDirLstatWithImpl(function<void(struct FUSE_STAT*)> implementation) {
  return CallLstatWithModeAndImpl(S_IFDIR, implementation);
}

struct stat FuseLstatTest::CallLstatWithImpl(function<void(struct FUSE_STAT*)> implementation) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).WillRepeatedly(Invoke([implementation](const char*, struct FUSE_STAT *stat) {
    implementation(stat);
  }));

  struct stat result{};
  LstatPath(FILENAME, &result);

  return result;
}

struct stat FuseLstatTest::CallLstatWithModeAndImpl(mode_t mode, function<void(struct FUSE_STAT*)> implementation) {
  return CallLstatWithImpl([mode, implementation] (struct FUSE_STAT *stat) {
    stat->st_mode = mode;
    implementation(stat);
  });
}
