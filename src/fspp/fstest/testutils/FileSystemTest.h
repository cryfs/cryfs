#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_

#include <gtest/gtest.h>
#include <type_traits>
#include <boost/static_assert.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <cpp-utils/system/stat.h>

#include "../../fs_interface/Device.h"
#include "../../fs_interface/Node.h"
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

  cpputils::unique_ref<fspp::Node> Load(const boost::filesystem::path &path) {
    auto loaded = device->Load(path);
    EXPECT_NE(boost::none, loaded);
    return std::move(*loaded);
  }

  cpputils::unique_ref<fspp::Dir> LoadDir(const boost::filesystem::path &path) {
	auto loaded = device->LoadDir(path);
    EXPECT_NE(boost::none, loaded);
	return std::move(*loaded);
  }

  cpputils::unique_ref<fspp::File> LoadFile(const boost::filesystem::path &path) {
	auto loaded = device->LoadFile(path);
    EXPECT_NE(boost::none, loaded);
    return std::move(*loaded);
  }

  cpputils::unique_ref<fspp::Symlink> LoadSymlink(const boost::filesystem::path &path) {
    auto loaded = device->LoadSymlink(path);
    EXPECT_NE(boost::none, loaded);
    return std::move(*loaded);
  }

  cpputils::unique_ref<fspp::Dir> CreateDir(const boost::filesystem::path &path) {
    this->LoadDir(path.parent_path())->createDir(path.filename().native(), this->MODE_PUBLIC, 0, 0);
    return this->LoadDir(path);
  }

  cpputils::unique_ref<fspp::File> CreateFile(const boost::filesystem::path &path) {
    this->LoadDir(path.parent_path())->createAndOpenFile(path.filename().native(), this->MODE_PUBLIC, 0, 0);
    return this->LoadFile(path);
  }

  cpputils::unique_ref<fspp::Symlink> CreateSymlink(const boost::filesystem::path &path, const boost::filesystem::path &target = "/my/symlink/target") {
    this->LoadDir(path.parent_path())->createSymlink(path.filename().native(), target, 0, 0);
    return this->LoadSymlink(path);
  }

  void EXPECT_IS_FILE(const cpputils::unique_ref<fspp::Node> &node) {
    EXPECT_NE(nullptr, dynamic_cast<const fspp::File*>(node.get()));
  }

  void EXPECT_IS_DIR(const cpputils::unique_ref<fspp::Node> &node) {
    EXPECT_NE(nullptr, dynamic_cast<const fspp::Dir*>(node.get()));
  }

  void EXPECT_IS_SYMLINK(const cpputils::unique_ref<fspp::Node> &node) {
    EXPECT_NE(nullptr, dynamic_cast<const fspp::Symlink*>(node.get()));
  }

  void setModificationTimestampLaterThanAccessTimestamp(const boost::filesystem::path& path) {
    auto node = device->Load(path).value();
    struct stat st{};
    node->stat(&st);
    st.st_mtim.tv_nsec = st.st_mtim.tv_nsec + 1;
    node->utimens(
            st.st_atim,
            st.st_mtim
    );
  }
};


#endif
