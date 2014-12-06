#pragma once
#ifndef TEST_FSPP_FUSE_FLUSH_TESTUTILS_FUSEFLUSHTEST_H_
#define TEST_FSPP_FUSE_FLUSH_TESTUTILS_FUSEFLUSHTEST_H_

#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "test/testutils/FuseTest.h"

class FuseFlushTest: public FuseTest {
public:
  const std::string FILENAME = "/myfile";

  void OpenAndCloseFile(const std::string &filename);
  int OpenFile(const TempTestFS *fs, const std::string &filename);
  void CloseFile(int fd);
};


#endif
