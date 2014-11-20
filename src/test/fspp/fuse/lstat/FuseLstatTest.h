#pragma once
#ifndef TEST_FSPP_FUSE_LSTAT_FUSELSTATTEST_H_
#define TEST_FSPP_FUSE_LSTAT_FUSELSTATTEST_H_

#include "gmock/gmock.h"

#include <string>
#include <functional>
#include <sys/stat.h>

#include "test/testutils/FuseTest.h"

class FuseLstatTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void LstatPath(const std::string &path) {
    struct stat dummy;
    LstatPath(path, &dummy);
  }

  void LstatPath(const std::string &path, struct stat *result) {
    auto fs = TestFS();

    auto realpath = fs->mountDir() / path;
    int retval = ::lstat(realpath.c_str(), result);
    EXPECT_EQ(0, retval) << "lstat syscall failed. errno: " << errno;
  }

protected:
  struct stat CallFileLstatWithImpl(std::function<void(struct stat*)> implementation) {
    return CallLstatWithModeAndImpl(S_IFREG, implementation);
  }

  struct stat CallDirLstatWithImpl(std::function<void(struct stat*)> implementation) {
    return CallLstatWithModeAndImpl(S_IFDIR, implementation);
  }

  struct stat CallLstatWithImpl(std::function<void(struct stat*)> implementation) {
    EXPECT_CALL(fsimpl, lstat(::testing::StrEq(FILENAME), ::testing::_)).WillRepeatedly(::testing::Invoke([implementation](const char*, struct ::stat *stat) {
      implementation(stat);
    }));

    struct stat result;
    LstatPath(FILENAME, &result);

    return result;
  }

private:

  struct stat CallLstatWithModeAndImpl(mode_t mode, std::function<void(struct stat*)> implementation) {
    return CallLstatWithImpl([mode, implementation] (struct stat *stat) {
      stat->st_mode = mode;
      implementation(stat);
    });
  }
};

#endif
