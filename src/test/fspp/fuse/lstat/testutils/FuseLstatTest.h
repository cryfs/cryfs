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

  void LstatPath(const std::string &path);
  void LstatPath(const std::string &path, struct stat *result);

protected:
  struct stat CallFileLstatWithImpl(std::function<void(struct stat*)> implementation);
  struct stat CallDirLstatWithImpl(std::function<void(struct stat*)> implementation);
  struct stat CallLstatWithImpl(std::function<void(struct stat*)> implementation);

private:

  struct stat CallLstatWithModeAndImpl(mode_t mode, std::function<void(struct stat*)> implementation);
};

#endif
