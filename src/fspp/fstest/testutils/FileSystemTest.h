#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_

#include <gtest/gtest.h>
#include <type_traits>
#include <boost/static_assert.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <cpp-utils/system/stat.h>
#include <fspp/impl/FilesystemImpl.h>

#include "../../fs_interface/Device.h"
#include "../../fs_interface/Node.h"
#include "../../fs_interface/Dir.h"
#include "../../fs_interface/File.h"
#include "../../fs_interface/Symlink.h"
#include "../../fs_interface/OpenFile.h"

class FileSystemTestFixture {
public:
  virtual ~FileSystemTestFixture() = default;
  virtual cpputils::unique_ref<fspp::Device> createDevice() = 0;
};

template<class ConcreteFileSystemTestFixture>
class FileSystemTest: public ::testing::Test {
public:
  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<FileSystemTestFixture, ConcreteFileSystemTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) FileSystemTest must inherit from FileSystemTestFixture"
  );

  FileSystemTest(): fixture(), _tmpInvalidDevice(fixture.createDevice()), device(_tmpInvalidDevice.get()), filesystem(std::move(_tmpInvalidDevice)) {}

  ConcreteFileSystemTestFixture fixture;

  cpputils::unique_ref<fspp::Device> _tmpInvalidDevice;
  fspp::Device* device;
  fspp::FilesystemImpl filesystem;

  static constexpr fspp::mode_t MODE_PUBLIC = fspp::mode_t()
        .addUserReadFlag().addUserWriteFlag().addUserExecFlag()
        .addGroupReadFlag().addGroupWriteFlag().addGroupExecFlag()
        .addOtherReadFlag().addOtherWriteFlag().addOtherExecFlag();

  cpputils::unique_ref<fspp::Node> Load(const boost::filesystem::path &path) {
    auto loaded = device->Load(path);
    EXPECT_NE(boost::none, loaded);
    return std::move(*loaded);
  }

  bool BlobExists(const blockstore::BlockId &id) {
    return device->BlobExists(id);
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
    this->LoadDir(path.parent_path())->createDir(path.filename().string(), this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    return this->LoadDir(path);
  }

  cpputils::unique_ref<fspp::File> CreateFile(const boost::filesystem::path &path) {
    this->LoadDir(path.parent_path())->createAndOpenFile(path.filename().string(), this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    return this->LoadFile(path);
  }



  cpputils::unique_ref<fspp::Symlink> CreateSymlink(const boost::filesystem::path &path, const boost::filesystem::path &target = "/my/symlink/target") {
    this->LoadDir(path.parent_path())->createSymlink(path.filename().string(), target, fspp::uid_t(0), fspp::gid_t(0));
    return this->LoadSymlink(path);
  }

  bool IsFileInDir(const boost::filesystem::path &path) {
    auto dir = LoadDir(path.parent_path());
    auto children = dir->children();
    auto it = std::find_if(children.begin(), children.end(), [path](const fspp::Dir::Entry& e) {return e.name == path.filename().string();});
    return (it != children.end() && it->type == fspp::Dir::EntryType::FILE);
  }

  bool IsDirInDir(const boost::filesystem::path &path) {
    auto dir = LoadDir(path.parent_path());
    auto children = dir->children();
    auto it = std::find_if(children.begin(), children.end(), [path](const fspp::Dir::Entry& e) {return e.name == path.filename().string();});
    return (it != children.end() && it->type == fspp::Dir::EntryType::DIR);
  }

  bool IsSymlinkInDir(const boost::filesystem::path &path) {
    auto dir = LoadDir(path.parent_path());
    auto children = dir->children();
    auto it = std::find_if(children.begin(), children.end(), [path](const fspp::Dir::Entry& e) {return e.name == path.filename().string();});
    return (it != children.end() && it->type == fspp::Dir::EntryType::SYMLINK);
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

  void EXCPECT_NLINKS(const boost::filesystem::path& path, uint32_t expectedLinks) {
    auto nod = Load(path);
    EXPECT_EQ(nod->stat().nlink, expectedLinks);
  }

  void setModificationTimestampLaterThanAccessTimestamp(const boost::filesystem::path& path) {
    auto node = device->Load(path).value();
    auto st = node->stat();
    st.mtime.tv_nsec = st.mtime.tv_nsec + 1;
    node->utimens(
            st.atime,
            st.mtime
    );
  }
};
template<class ConcreteFileSystemTestFixture> constexpr fspp::mode_t FileSystemTest<ConcreteFileSystemTestFixture>::MODE_PUBLIC;


#endif
