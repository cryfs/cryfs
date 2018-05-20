#include <gtest/gtest.h>

#include "cpp-utils/tempfile/TempFile.h"
#include "cpp-utils/tempfile/TempDir.h"

#include <fstream>

using ::testing::Test;
using std::ifstream;
using std::ofstream;

using namespace cpputils;

namespace bf = boost::filesystem;

class TempFileTest: public Test {
public:
  TempFileTest(): tempdir(), filepath_sample(tempdir.path() / "myfile") {}

  TempDir tempdir;
  bf::path filepath_sample;

  void CreateFile(const bf::path &path) {
    ofstream file(path.string().c_str());
  }
};

TEST_F(TempFileTest, FileIsCreated) {
  TempFile file;
  EXPECT_TRUE(bf::exists(file.path()));
  EXPECT_TRUE(bf::is_regular_file(file.path()));
}

TEST_F(TempFileTest, FileIsReadable) {
  TempFile file;
  ifstream opened(file.path().string().c_str());
  EXPECT_TRUE(opened.good());
}

TEST_F(TempFileTest, FileIsCreatedEmpty) {
  TempFile file;
  ifstream opened(file.path().string().c_str());
  opened.get();
  EXPECT_TRUE(opened.eof());
}

TEST_F(TempFileTest, FileIsWriteable) {
  TempFile file;
  ofstream opened(file.path().string().c_str());
  EXPECT_TRUE(opened.good());
}

TEST_F(TempFileTest, FileIsDeletedAfterUse) {
  bf::path filepath;
  {
    TempFile file;
    filepath = file.path();
  }
  EXPECT_FALSE(bf::exists(filepath));
}

TEST_F(TempFileTest, DontCreateFileSpecified_FileIsNotCreated) {
  TempFile file(false);
  EXPECT_FALSE(bf::exists(file.path()));
}

TEST_F(TempFileTest, DontCreateFileSpecified_FileIsCreatable) {
  TempFile file(false);
  CreateFile(file.path());
  EXPECT_TRUE(bf::exists(file.path()));
}

TEST_F(TempFileTest, DontCreateFileSpecified_FileIsDeletedAfterUse) {
  bf::path filepath;
  {
    TempFile file(false);
    CreateFile(file.path());
    filepath = file.path();
  }
  EXPECT_FALSE(bf::exists(filepath));
}

TEST_F(TempFileTest, PathGiven_FileIsCreatedAtGivenPath) {
  TempFile file(filepath_sample);
  EXPECT_EQ(filepath_sample, file.path());
}

TEST_F(TempFileTest, PathGiven_FileIsCreatedAndAccessible) {
  TempFile file(filepath_sample);
  EXPECT_TRUE(bf::exists(filepath_sample));
}

TEST_F(TempFileTest, PathGiven_FileIsDeletedAfterUse) {
  {
    TempFile file(filepath_sample);
  }
  EXPECT_FALSE(bf::exists(filepath_sample));
}

TEST_F(TempFileTest, PathGiven_DontCreateFileSpecified_FileIsNotCreated) {
  TempFile file(filepath_sample, false);
  EXPECT_FALSE(bf::exists(filepath_sample));
}

TEST_F(TempFileTest, PathGiven_DontCreateFileSpecified_FileIsCreatable) {
  TempFile file(filepath_sample, false);
  CreateFile(filepath_sample);
  EXPECT_TRUE(bf::exists(filepath_sample));
}

TEST_F(TempFileTest, PathGiven_DontCreateFileSpecified_FileIsDeletedAfterUse) {
  {
    TempFile file(filepath_sample, false);
    CreateFile(filepath_sample);
  }
  EXPECT_FALSE(bf::exists(filepath_sample));
}
