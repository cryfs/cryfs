#include "FuseReadDirTest.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::vector;
using std::string;

using ::testing::Action;
using ::testing::Return;

unique_ref<vector<string>> FuseReadDirTest::ReadDir(const char *dirname) {
  auto fs = TestFS();

  DIR *dir = openDir(fs.get(), dirname);

  auto result = make_unique_ref<vector<string>>();
  readDirEntries(dir, result.get());
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
  int error = readDirEntriesAllowError(dir, result.get());
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
  int error = readDirEntriesAllowError(dir, result);
  EXPECT_EQ(0, error);
}

int FuseReadDirTest::readDirEntriesAllowError(DIR *dir, vector<string> *result) {
  struct dirent *entry = nullptr;
  int error = readNextDirEntryAllowError(dir, &entry);
  if (error != 0) {
    return error;
  }
  while(entry != nullptr) {
    result->push_back(entry->d_name);
    int error = readNextDirEntryAllowError(dir, &entry);
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
  int retval = ::closedir(dir);
  EXPECT_EQ(0, retval) << "Closing dir failed";
}

Action<vector<fspp::Dir::Entry>*(const char*)> FuseReadDirTest::ReturnDirEntries(vector<std::string> entries) {
  vector<fspp::Dir::Entry> *direntries = new vector<fspp::Dir::Entry>(entries.size(), fspp::Dir::Entry(fspp::Dir::EntryType::FILE, ""));
  for(size_t i = 0; i < entries.size(); ++i) {
    (*direntries)[i].name = entries[i];
  }
  return Return(direntries);
}
