#include "FuseReadDirTest.h"
#include "fspp/fs_interface/Dir.h"
#include <boost/filesystem/path.hpp>
#include <cerrno>
#include <cstddef>
#include <dirent.h>
#include <gtest/gtest.h>
#include <string>
#include <utility>

using cpputils::make_unique_ref;
using std::vector;
using std::string;

using ::testing::Action;
using ::testing::Return;

vector<string> FuseReadDirTest::ReadDir(const char *dirname) {
  auto fs = TestFS();

  DIR *dir = openDir(fs.get(), dirname);

  vector<string> result;
  readDirEntries(dir, &result);
  closeDir(dir);
  return result;
}

int FuseReadDirTest::ReadDirReturnError(const char *dirname) {
  auto fs = TestFS();

  errno = 0;
  DIR *dir = openDirAllowError(fs.get(), dirname);
  EXPECT_EQ(errno!=0, dir==nullptr) << "errno should exactly be != 0 if opendir returned nullptr";
  if (errno != 0) {
    return errno;
  }

  auto result = make_unique_ref<vector<string>>();
  const int error = readDirEntriesAllowError(dir, result.get());
  closeDir(dir);
  return error;
}

DIR *FuseReadDirTest::openDir(TempTestFS *fs, const char *dirname) {
  DIR *dir = openDirAllowError(fs, dirname);
  EXPECT_NE(nullptr, dir) << "Opening directory failed";
  return dir;
}

DIR *FuseReadDirTest::openDirAllowError(TempTestFS *fs, const char *dirname) {
  auto realpath = fs->mountDir() / dirname;
  return ::opendir(realpath.string().c_str());
}

void FuseReadDirTest::readDirEntries(DIR *dir, vector<string> *result) {
  const int error = readDirEntriesAllowError(dir, result);
  EXPECT_EQ(0, error);
}

int FuseReadDirTest::readDirEntriesAllowError(DIR *dir, vector<string> *result) {
  struct dirent *entry = nullptr;
  const int error = readNextDirEntryAllowError(dir, &entry);
  if (error != 0) {
    return error;
  }
  while(entry != nullptr) {
    result->push_back(entry->d_name);
    const int error = readNextDirEntryAllowError(dir, &entry);
    if (error != 0) {
      return error;
    }
  }
  return 0;
}

int FuseReadDirTest::readNextDirEntryAllowError(DIR *dir, struct dirent **result) {
  errno = 0;
  *result = ::readdir(dir);
  return errno;
}

void FuseReadDirTest::closeDir(DIR *dir) {
  const int retval = ::closedir(dir);
  EXPECT_EQ(0, retval) << "Closing dir failed";
}

Action<vector<fspp::Dir::Entry>(const boost::filesystem::path&)> FuseReadDirTest::ReturnDirEntries(vector<std::string> entries) {
  vector<fspp::Dir::Entry> direntries(entries.size(), fspp::Dir::Entry(fspp::Dir::EntryType::FILE, ""));
  for(size_t i = 0; i < entries.size(); ++i) {
    direntries[i].name = entries[i];
  }
  return Return(std::move(direntries));
}
