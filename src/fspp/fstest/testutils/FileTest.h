#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILETEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILETEST_H_

#include "FileSystemTest.h"
#include <cpp-utils/data/Data.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/system/stat.h>

template<class ConcreteFileSystemTestFixture>
class FileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  FileTest() {
		this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, 0, 0);

		this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, 0, 0);
		this->LoadDir("/mydir")->createAndOpenFile("mynestedfile", this->MODE_PUBLIC, 0, 0);

		this->LoadDir("/")->createDir("mydir2", this->MODE_PUBLIC, 0, 0);
  }

  std::function<cpputils::unique_ref<fspp::File>()> file_root() {
	  return [this]{ return this->LoadFile("/myfile"); };
  }

  std::function<cpputils::unique_ref<fspp::Node>()> file_root_node() {
	  return [this]{return this->Load("/myfile"); };
  }

  std::function<cpputils::unique_ref<fspp::File>()> file_nested() {
	  return [this]{ return this->LoadFile("/mydir/mynestedfile"); };
  }

  std::function<cpputils::unique_ref<fspp::Node>()> file_nested_node() {
	  return [this]{ return this->Load("/mydir/mynestedfile"); };
  }

  //TODO IN_STAT still needed after moving it to FsppNodeTest?
  void IN_STAT(std::function<cpputils::unique_ref<fspp::File>()> file, std::function<cpputils::unique_ref<fspp::Node>()> node, std::function<void (struct stat)> callback) {
	  struct stat st1{}, st2{};
	  node()->stat(&st1);
	  callback(st1);
	  file()->open(O_RDONLY)->stat(&st2);
	  callback(st2);
  }

  void EXPECT_SIZE(uint64_t expectedSize, std::function<cpputils::unique_ref<fspp::File>()> file, std::function<cpputils::unique_ref<fspp::Node>()> node) {
		IN_STAT(file, node, [expectedSize] (struct stat st) {
			EXPECT_EQ(expectedSize, static_cast<uint64_t>(st.st_size));
		});

		EXPECT_NUMBYTES_READABLE(expectedSize, file);
  }

  void EXPECT_NUMBYTES_READABLE(uint64_t expectedSize, std::function<cpputils::unique_ref<fspp::File>()> file) {
		auto openFile = file()->open(O_RDONLY);
		cpputils::Data data(expectedSize);
		//Try to read one byte more than the expected size
		ssize_t readBytes = openFile->read(data.data(), expectedSize+1, 0);
		//and check that it only read the expected size (but also not less)
		EXPECT_EQ(expectedSize, static_cast<uint64_t>(readBytes));
  }

  void EXPECT_ATIME_EQ(struct timespec expected, struct stat st) {
	  EXPECT_EQ(expected.tv_sec, st.st_atim.tv_sec);
	  EXPECT_EQ(expected.tv_nsec, st.st_atim.tv_nsec);
  }

  void EXPECT_MTIME_EQ(struct timespec expected, struct stat st) {
      EXPECT_EQ(expected.tv_sec, st.st_mtim.tv_sec);
      EXPECT_EQ(expected.tv_nsec, st.st_mtim.tv_nsec);
  }
};

#endif
