#include <boost/filesystem/directory.hpp>
#include <boost/filesystem/operations.hpp>
#include <boost/filesystem/path.hpp>
#include <gtest/gtest.h>

#include "cpp-utils/tempfile/TempDir.h"

#include <fstream>

using ::testing::Test;
using std::ofstream;

using namespace cpputils;

namespace bf = boost::filesystem;

class TempDirTest: public Test {
public:
  void EXPECT_ENTRY_COUNT(int expected, const bf::path &path) {
    const int actual = CountEntries(path);
    EXPECT_EQ(expected, actual);
  }

  int CountEntries(const bf::path &path) {
    int count = 0;
    for (bf::directory_iterator iter(path); iter != bf::directory_iterator(); ++iter) {
      ++count;
    }
    return count;
  }

  void CreateFile(const bf::path &path) {
    const ofstream file(path.string().c_str());
  }
};

TEST_F(TempDirTest, DirIsCreated) {
  const TempDir dir;
  EXPECT_TRUE(bf::exists(dir.path()));
  EXPECT_TRUE(bf::is_directory(dir.path()));
}

TEST_F(TempDirTest, DirIsCreatedEmpty) {
  const TempDir dir;
  EXPECT_ENTRY_COUNT(0, dir.path());
}

TEST_F(TempDirTest, DirIsWriteable) {
  const TempDir dir;
  CreateFile(dir.path() / "myfile");
  EXPECT_ENTRY_COUNT(1, dir.path());
}

TEST_F(TempDirTest, DirIsDeletedAfterUse) {
  bf::path dirpath;
  {
    const TempDir dir;
    dirpath = dir.path();
  }
  EXPECT_FALSE(bf::exists(dirpath));
}
