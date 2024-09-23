#include "FuseStatfsTest.h"
#include "gmock/gmock.h"
#include <cerrno>
#include <gtest/gtest.h>
#include <string>
#include <sys/statvfs.h>

using std::function;
using ::testing::Invoke;

void FuseStatfsTest::Statfs(const std::string &path) {
  struct ::statvfs dummy{};
  Statfs(path, &dummy);
}

int FuseStatfsTest::StatfsReturnError(const std::string &path) {
  struct ::statvfs dummy{};
  return StatfsReturnError(path, &dummy);
}

void FuseStatfsTest::Statfs(const std::string &path, struct ::statvfs *result) {
  const int error = StatfsReturnError(path, result);
  EXPECT_EQ(0, error) << "lstat syscall failed. errno: " << errno;
}

int FuseStatfsTest::StatfsReturnError(const std::string &path, struct ::statvfs *result) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / path;
  const int retval = ::statvfs(realpath.string().c_str(), result);
  if (retval == 0) {
    return 0;
  } else {
    return errno;
  }
}

struct ::statvfs FuseStatfsTest::CallStatfsWithImpl(function<void(struct ::statvfs*)> implementation) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, statfs(testing::_)).WillRepeatedly(Invoke([implementation](struct ::statvfs *stat) {
    implementation(stat);
  }));

  struct ::statvfs result{};
  Statfs(FILENAME, &result);

  return result;
}

