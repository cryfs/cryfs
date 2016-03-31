#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_

#include <gtest/gtest.h>
#include <type_traits>
#include <boost/static_assert.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

#include "../../fs_interface/Device.h"
#include "../../fs_interface/Dir.h"
#include "../../fs_interface/File.h"
#include "../../fs_interface/Symlink.h"
#include "../../fs_interface/OpenFile.h"

class FileSystemTestFixture {
public:
  virtual ~FileSystemTestFixture() {}
  virtual cpputils::unique_ref<fspp::Device> createDevice() = 0;
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
  cpputils::unique_ref<fspp::Device> device;

  static constexpr mode_t MODE_PUBLIC = S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IWGRP | S_IXGRP | S_IROTH | S_IWOTH | S_IXOTH;

  cpputils::unique_ref<fspp::Dir> LoadDir(const boost::filesystem::path &path) {
	auto loaded = device->Load(path);
    EXPECT_NE(boost::none, loaded);
	auto dir = cpputils::dynamic_pointer_move<fspp::Dir>(*loaded);
	EXPECT_NE(boost::none, dir);
	return std::move(*dir);
  }

  cpputils::unique_ref<fspp::File> LoadFile(const boost::filesystem::path &path) {
	auto loaded = device->Load(path);
    EXPECT_NE(boost::none, loaded);
	auto file = cpputils::dynamic_pointer_move<fspp::File>(*loaded);
	EXPECT_NE(boost::none, file);
	return std::move(*file);
  }

  cpputils::unique_ref<fspp::Symlink> LoadSymlink(const boost::filesystem::path &path) {
    auto loaded = device->Load(path);
    EXPECT_NE(boost::none, loaded);
    auto symlink = cpputils::dynamic_pointer_move<fspp::Symlink>(*loaded);
    EXPECT_NE(boost::none, symlink);
    return std::move(*symlink);
  }

  cpputils::unique_ref<fspp::Dir> CreateDir(const boost::filesystem::path &path) {
    this->LoadDir(path.parent_path())->createDir(path.filename().native(), this->MODE_PUBLIC, 0, 0);
    return this->LoadDir(path);
  }

  cpputils::unique_ref<fspp::File> CreateFile(const boost::filesystem::path &path) {
    this->LoadDir(path.parent_path())->createAndOpenFile(path.filename().native(), this->MODE_PUBLIC, 0, 0);
    return this->LoadFile(path);
  }

  cpputils::unique_ref<fspp::Symlink> CreateSymlink(const boost::filesystem::path &path) {
    this->LoadDir(path.parent_path())->createSymlink(path.filename().native(), "/my/symlink/target", 0, 0);
    return this->LoadSymlink(path);
  }
};


#endif
