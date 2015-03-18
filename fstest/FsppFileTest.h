#ifndef MESSMER_FSPP_FSTEST_FSPPFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPFILETEST_H_

#include <sys/fcntl.h>
#include <sys/stat.h>

template<class ConcreteFileSystemTestFixture>
class FsppFileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  FsppFileTest() {
	this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
	file = this->LoadFile("/myfile");

	this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
	this->LoadDir("/mydir")->createAndOpenFile("mynestedfile", this->MODE_PUBLIC);
	file_nested = this->LoadFile("/mydir/mynestedfile");
  }
  std::unique_ptr<fspp::File> file;
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
	blockstore::Data data(expectedSize);
	//Try to read one byte more than the expected size
	ssize_t readBytes = file.read(data.data(), expectedSize+1, 0);
	//and check that it only read the expected size (but also not less)
	EXPECT_EQ(expectedSize, readBytes);
  }
};

TYPED_TEST_CASE_P(FsppFileTest);

//TODO Right now, each test case is there twice (normal and _Nested). Find better solution without code duplication.

TYPED_TEST_P(FsppFileTest, Open_RDONLY) {
  this->file->open(O_RDONLY);
}

TYPED_TEST_P(FsppFileTest, Open_RDONLY_Nested) {
  this->file_nested->open(O_RDONLY);
}

TYPED_TEST_P(FsppFileTest, Open_WRONLY) {
  this->file->open(O_WRONLY);
}

TYPED_TEST_P(FsppFileTest, Open_WRONLY_Nested) {
  this->file_nested->open(O_WRONLY);
}

TYPED_TEST_P(FsppFileTest, Open_RDWR) {
  this->file->open(O_RDWR);
}

TYPED_TEST_P(FsppFileTest, Open_RDWR_Nested) {
  this->file_nested->open(O_RDWR);
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange1) {
  this->file->truncate(0);
  this->EXPECT_SIZE(0, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange1_Nested) {
  this->file->truncate(0);
  this->EXPECT_SIZE(0, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_GrowTo1) {
  this->file->truncate(1);
  this->EXPECT_SIZE(1, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_GrowTo1_Nested) {
  this->file->truncate(1);
  this->EXPECT_SIZE(1, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_Grow) {
  this->file->truncate(10*1024*1024);
  this->EXPECT_SIZE(10*1024*1024, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_Grow_Nested) {
  this->file->truncate(10*1024*1024);
  this->EXPECT_SIZE(10*1024*1024, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange2) {
  this->file->truncate(10*1024*1024);
  this->file->truncate(10*1024*1024);
  this->EXPECT_SIZE(10*1024*1024, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange2_Nested) {
  this->file->truncate(10*1024*1024);
  this->file->truncate(10*1024*1024);
  this->EXPECT_SIZE(10*1024*1024, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_Shrink) {
  this->file->truncate(10*1024*1024);
  this->file->truncate(5*1024*1024);
  this->EXPECT_SIZE(5*1024*1024, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_Shrink_Nested) {
  this->file->truncate(10*1024*1024);
  this->file->truncate(5*1024*1024);
  this->EXPECT_SIZE(5*1024*1024, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_ShrinkTo0) {
  this->file->truncate(10*1024*1024);
  this->file->truncate(0);
  this->EXPECT_SIZE(0, *this->file);
}

TYPED_TEST_P(FsppFileTest, Truncate_ShrinkTo0_Nested) {
  this->file->truncate(10*1024*1024);
  this->file->truncate(0);
  this->EXPECT_SIZE(0, *this->file);
}

TYPED_TEST_P(FsppFileTest, Stat_CreatedFileIsEmpty) {
  this->EXPECT_SIZE(0, *this->file);
}

TYPED_TEST_P(FsppFileTest, Stat_CreatedFileIsEmpty_Nested) {
  this->EXPECT_SIZE(0, *this->file_nested);
}

REGISTER_TYPED_TEST_CASE_P(FsppFileTest,
  Open_RDONLY,
  Open_RDONLY_Nested,
  Open_WRONLY,
  Open_WRONLY_Nested,
  Open_RDWR,
  Open_RDWR_Nested,
  Truncate_DontChange1,
  Truncate_DontChange1_Nested,
  Truncate_GrowTo1,
  Truncate_GrowTo1_Nested,
  Truncate_Grow,
  Truncate_Grow_Nested,
  Truncate_DontChange2,
  Truncate_DontChange2_Nested,
  Truncate_Shrink,
  Truncate_Shrink_Nested,
  Truncate_ShrinkTo0,
  Truncate_ShrinkTo0_Nested,
  Stat_CreatedFileIsEmpty,
  Stat_CreatedFileIsEmpty_Nested
);

//TODO unlink
//TODO stat
//TODO access
//TODO rename
//TODO utimens

#endif
