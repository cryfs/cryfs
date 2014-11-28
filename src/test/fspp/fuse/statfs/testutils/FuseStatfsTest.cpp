#include "FuseStatfsTest.h"

using std::function;
using ::testing::StrEq;
using ::testing::_;
using ::testing::Invoke;

void FuseStatfsTest::Statfs(const std::string &path) {
  struct ::statvfs dummy;
  Statfs(path, &dummy);
}

int FuseStatfsTest::StatfsAllowErrors(const std::string &path) {
  struct ::statvfs dummy;
  return StatfsAllowErrors(path, &dummy);
}

void FuseStatfsTest::Statfs(const std::string &path, struct ::statvfs *result) {
  int retval = StatfsAllowErrors(path, result);
  EXPECT_EQ(0, retval) << "lstat syscall failed. errno: " << errno;
}

int FuseStatfsTest::StatfsAllowErrors(const std::string &path, struct ::statvfs *result) {
  auto fs = TestFS();

  auto realpath = fs->mountDir() / path;
  return ::statvfs(realpath.c_str(), result);
}

struct ::statvfs FuseStatfsTest::CallStatfsWithImpl(function<void(struct ::statvfs*)> implementation) {
  ReturnIsFileOnLstat(FILENAME);
  EXPECT_CALL(fsimpl, statfs(StrEq(FILENAME), _)).WillRepeatedly(Invoke([implementation](const char*, struct ::statvfs *stat) {
    implementation(stat);
  }));

  struct ::statvfs result;
  Statfs(FILENAME, &result);

  return result;
}

