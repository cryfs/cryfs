#pragma once
#ifndef TEST_FSPP_FUSE_READDIR_TESTUTILS_FUSEREADDIRTEST_H_
#define TEST_FSPP_FUSE_READDIR_TESTUTILS_FUSEREADDIRTEST_H_

#include "test/testutils/FuseTest.h"
#include <dirent.h>

class FuseReadDirTest: public FuseTest {
public:
  const char *DIRNAME = "/mydir";

  std::unique_ptr<std::vector<std::string>> ReadDir(const char *dirname);
  int ReadDirReturnError(const char *dirname);

  static ::testing::Action<std::vector<std::string>*(const char*)> ReturnDirEntries(std::vector<std::string> entries);

private:
  DIR *openDir(TempTestFS *fs, const char *dirname);
  DIR *openDirAllowError(TempTestFS *fs, const char *dirname);
  void readDirEntries(DIR *dir, std::vector<std::string> *result);
  int readDirEntriesAllowError(DIR *dir, std::vector<std::string> *result);
  int readNextDirEntryAllowError(DIR *dir, struct dirent **result);
  void closeDir(DIR *dir);
};

#endif
