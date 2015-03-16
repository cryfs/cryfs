#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILESYSTEMTEST_H_

#include <google/gtest/gtest.h>
#include <memory>
#include <type_traits>
#include <boost/static_assert.hpp>
#include <messmer/cpp-utils/pointer.h>

#include "../../fs_interface/Device.h"
#include "../../fs_interface/Dir.h"
#include "../../fs_interface/File.h"
#include "../../fs_interface/OpenFile.h"

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

  static constexpr mode_t MODE_PUBLIC = S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IWGRP | S_IXGRP | S_IROTH | S_IWOTH | S_IXOTH;

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


#endif
