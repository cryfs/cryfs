#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILETEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILETEST_H_

#include "FileSystemTest.h"
#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>

template<class ConcreteFileSystemTestFixture>
class FileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  FileTest() {
	this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, 0, 0);
	file_root = cpputils::to_unique_ptr(this->LoadFile("/myfile"));

	this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, 0, 0);
	this->LoadDir("/mydir")->createAndOpenFile("mynestedfile", this->MODE_PUBLIC, 0, 0);
	file_nested = cpputils::to_unique_ptr(this->LoadFile("/mydir/mynestedfile"));
  }
  std::unique_ptr<fspp::File> file_root;
  std::unique_ptr<fspp::File> file_nested;

  void EXPECT_SIZE(uint64_t expectedSize, const fspp::File &file) {
	EXPECT_SIZE_IN_FILE(expectedSize, file);
	auto openFile = file.open(O_RDONLY);
	EXPECT_SIZE_IN_OPEN_FILE(expectedSize, *openFile);
	EXPECT_NUMBYTES_READABLE(expectedSize, *openFile);
  }

  void EXPECT_SIZE_IN_FILE(uint64_t expectedSize, const fspp::File &file) {
	struct stat st;
	file.stat(&st);
    EXPECT_EQ(expectedSize, st.st_size);
  }

  void EXPECT_SIZE_IN_OPEN_FILE(uint64_t expectedSize, const fspp::OpenFile &file) {
	struct stat st;
	file.stat(&st);
    EXPECT_EQ(expectedSize, st.st_size);
  }

  void EXPECT_NUMBYTES_READABLE(uint64_t expectedSize, const fspp::OpenFile &file) {
	cpputils::Data data(expectedSize);
	//Try to read one byte more than the expected size
	ssize_t readBytes = file.read(data.data(), expectedSize+1, 0);
	//and check that it only read the expected size (but also not less)
	EXPECT_EQ(expectedSize, readBytes);
  }
};

#endif
