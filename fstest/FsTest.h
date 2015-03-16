#ifndef MESSMER_FSPP_FSTEST_FSTEST_H_
#define MESSMER_FSPP_FSTEST_FSTEST_H_

#include <memory>
#include <google/gtest/gtest.h>
#include <boost/static_assert.hpp>
#include <type_traits>
#include <messmer/cpp-utils/pointer.h>

#include "../fs_interface/Device.h"
#include "../fs_interface/Dir.h"
#include "../fs_interface/File.h"
#include "../fs_interface/OpenFile.h"

class FileSystemTestFixture {
public:
  virtual std::unique_ptr<fspp::Device> createDevice() = 0;
};

template<class ConcreteFileSystemTestFixture>
class FileSystemTest: public ::testing::Test {
public:
  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<FileSystemTestFixture, ConcreteFileSystemTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) FileSystemTest must inherit from FileSystemTestFixture"
  );

  FileSystemTest(): fixture(), device(fixture.createDevice()) {}

  ConcreteFileSystemTestFixture fixture;
  std::unique_ptr<fspp::Device> device;

  std::unique_ptr<fspp::Dir> LoadDir(const boost::filesystem::path &path) {
	auto loaded = device->Load(path);
	auto dir = cpputils::dynamic_pointer_move<fspp::Dir>(loaded);
	EXPECT_NE(nullptr, dir.get());
	return dir;
  }

  std::unique_ptr<fspp::File> LoadFile(const boost::filesystem::path &path) {
	auto loaded = device->Load(path);
	auto file = cpputils::dynamic_pointer_move<fspp::File>(loaded);
	EXPECT_NE(nullptr, file.get());
	return file;
  }
};

TYPED_TEST_CASE_P(FileSystemTest);

TYPED_TEST_P(FileSystemTest, InitFilesystem) {
  //fixture->createDevice() is called in the FileSystemTest constructor
}

TYPED_TEST_P(FileSystemTest, LoadRootDir) {
  this->LoadDir("/");
}

TYPED_TEST_P(FileSystemTest, LoadEntriesOfEmptyRootDir) {
  auto rootdir = this->LoadDir("/");
  auto children = rootdir->children();
  EXPECT_EQ(0, children->size());
}

TYPED_TEST_P(FileSystemTest, AddFileToRootDirAndLoadEntries) {
  auto rootdir = this->LoadDir("/");
  rootdir->createAndOpenFile("myfile", 0);
  auto children = rootdir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
}

TYPED_TEST_P(FileSystemTest, AddFileToRootDirAndLoadIt) {
  this->LoadDir("/")->createAndOpenFile("myfile", 0);
  this->LoadFile("/myfile");
}

TYPED_TEST_P(FileSystemTest, AddDirToRootDirAndLoadEntries) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("mydir", 0);
  auto children = rootdir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[0].type);
  EXPECT_EQ("mydir", (*children)[0].name);
}

TYPED_TEST_P(FileSystemTest, AddDirToRootDirAndLoadIt) {
  this->LoadDir("/")->createDir("mydir", 0);
  this->LoadDir("/mydir");
}

//TODO Group test cases to fspp functions (e.g. AddFileToRootDirAndLoadEntries to Dir::children(). AddFileToRootDirAndLoadIt to Device::Load and to Dir::createAndOpenFile.

//TODO Add File/Dir to subdir and check entries
//TODO Add File/Dir to subdir and load it using path
//TODO Add File/Dir to subsubdir and check entries
//TODO Add File/Dir to subsubdir and load it using path
//TODO Build dir structure with more than one entry

//TODO ...


//TODO statfs

REGISTER_TYPED_TEST_CASE_P(FileSystemTest,
  InitFilesystem,
  LoadRootDir,
  LoadEntriesOfEmptyRootDir,
  AddFileToRootDirAndLoadEntries,
  AddFileToRootDirAndLoadIt,
  AddDirToRootDirAndLoadEntries,
  AddDirToRootDirAndLoadIt
);


#endif
