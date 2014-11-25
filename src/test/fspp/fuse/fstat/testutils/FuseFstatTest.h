#pragma once
#ifndef TEST_FSPP_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_
#define TEST_FSPP_FUSE_FSTAT_TESTUTILS_FUSEFSTATTEST_H_

#include "test/testutils/FuseTest.h"

class FuseFstatTest: public FuseTest {
public:
  int CreateFile(const TempTestFS *fs, const std::string &filename) {
    int fd = CreateFileAllowErrors(fs, filename);
    EXPECT_GE(fd, 0) << "Opening file failed";
    return fd;
  }
  int CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename) {
    auto real_path = fs->mountDir() / filename;
    return ::open(real_path.c_str(), O_RDWR | O_CREAT);
  }
};


#endif
