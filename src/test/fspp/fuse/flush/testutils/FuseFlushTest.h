#pragma once
#ifndef TEST_FSPP_FUSE_FLUSH_TESTUTILS_FUSEFLUSHTEST_H_
#define TEST_FSPP_FUSE_FLUSH_TESTUTILS_FUSEFLUSHTEST_H_

#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "test/testutils/FuseTest.h"

class FuseFlushTest: public FuseTest {
public:
  const std::string FILENAME = "/myfile";

  void OpenAndCloseFile(const std::string &filename) {
    auto fs = TestFS();
    int fd = OpenFile(fs.get(), filename);
    CloseFile(fd);
  }

  int OpenFile(const TempTestFS *fs, const std::string &filename) {
    auto real_path = fs->mountDir() / filename;
    int fd = ::open(real_path.c_str(), O_RDONLY);
    EXPECT_GE(fd, 0) << "Opening file failed";
    return fd;
  }

  void CloseFile(int fd) {
    int retval = ::close(fd);
    EXPECT_EQ(0, retval);
  }
};


#endif
