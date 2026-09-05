#pragma once
#ifndef MESSMER_FSPP_TEST_FUSE_READDIR_TESTUTILS_FUSEREADDIRTEST_H_
#define MESSMER_FSPP_TEST_FUSE_READDIR_TESTUTILS_FUSEREADDIRTEST_H_

#include "../../../testutils/FuseTest.h"
#include <dirent.h>
#include <utility>
#include "fspp/fs_interface/Dir.h"

class FuseReadDirTest: public FuseTest {
public:
  const char *DIRNAME = "/mydir";

  std::vector<std::string> ReadDir(const char *dirname);
  // Reads the directory and returns each entry's name together with the d_type the kernel reported
  // for it. d_type is DT_UNKNOWN if the file system didn't tell the kernel what the entry is.
  std::vector<std::pair<std::string, unsigned char>> ReadDirEntryTypes(const char *dirname);
  int ReadDirReturnError(const char *dirname);

  static ::testing::Action<std::vector<fspp::Dir::Entry>(const boost::filesystem::path&)> ReturnDirEntries(std::vector<std::string> entries);

private:
  DIR *openDir(TempTestFS *fs, const char *dirname);
  DIR *openDirAllowError(TempTestFS *fs, const char *dirname);
  void readDirEntries(DIR *dir, std::vector<std::string> *result);
  int readDirEntriesAllowError(DIR *dir, std::vector<std::string> *result);
  int readNextDirEntryAllowError(DIR *dir, struct dirent **result);
  void closeDir(DIR *dir);
};

#endif
