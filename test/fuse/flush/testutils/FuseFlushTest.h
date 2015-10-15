#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_FLUSH_TESTUTILS_FUSEFLUSHTEST_H_
#define MESSMER_FSPP_TEST_FUSE_FLUSH_TESTUTILS_FUSEFLUSHTEST_H_

#include "../../../testutils/FuseTest.h"

class FuseFlushTest: public FuseTest {
public:
  const std::string FILENAME = "/myfile";

  void OpenAndCloseFile(const std::string &filename);
  int OpenFile(const TempTestFS *fs, const std::string &filename);
  void CloseFile(int fd);
};


#endif
