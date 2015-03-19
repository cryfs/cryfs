#ifndef MESSMER_FSPP_FSTEST_FSPPFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPFILETEST_H_

#include <sys/fcntl.h>
#include <sys/stat.h>

template<class ConcreteFileSystemTestFixture>
class FsppFileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  FsppFileTest() {
	this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
	file_root = this->LoadFile("/myfile");

	this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
	this->LoadDir("/mydir")->createAndOpenFile("mynestedfile", this->MODE_PUBLIC);
	file_nested = this->LoadFile("/mydir/mynestedfile");
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
	blockstore::Data data(expectedSize);
	//Try to read one byte more than the expected size
	ssize_t readBytes = file.read(data.data(), expectedSize+1, 0);
	//and check that it only read the expected size (but also not less)
	EXPECT_EQ(expectedSize, readBytes);
  }

  void Test_Open_RDONLY(fspp::File *file) {
    file->open(O_RDONLY);
  }

  void Test_Open_WRONLY(fspp::File *file) {
    file->open(O_WRONLY);
  }

  void Test_Open_RDWR(fspp::File *file) {
    file->open(O_RDONLY);
  }

  void Test_Truncate_DontChange1(fspp::File *file) {
	file->truncate(0);
	this->EXPECT_SIZE(0, *file);
  }

  void Test_Truncate_GrowTo1(fspp::File *file) {
	file->truncate(1);
	this->EXPECT_SIZE(1, *file);
  }

  void Test_Truncate_Grow(fspp::File *file) {
	file->truncate(10*1024*1024);
	this->EXPECT_SIZE(10*1024*1024, *file);
  }

  void Test_Truncate_DontChange2(fspp::File *file) {
	file->truncate(10*1024*1024);
	file->truncate(10*1024*1024);
	this->EXPECT_SIZE(10*1024*1024, *file);
  }

  void Test_Truncate_Shrink(fspp::File *file) {
    file->truncate(10*1024*1024);
    file->truncate(5*1024*1024);
    this->EXPECT_SIZE(5*1024*1024, *file);
  }

  void Test_Truncate_ShrinkTo0(fspp::File *file) {
	file->truncate(10*1024*1024);
	file->truncate(0);
	this->EXPECT_SIZE(0, *file);
  }

  void Test_Stat_CreatedFileIsEmpty(fspp::File *file) {
	this->EXPECT_SIZE(0, *file);
  }
};

TYPED_TEST_CASE_P(FsppFileTest);

TYPED_TEST_P(FsppFileTest, Open_RDONLY) {
  this->Test_Open_RDONLY(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Open_RDONLY_Nested) {
  this->Test_Open_RDONLY(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Open_WRONLY) {
  this->Test_Open_WRONLY(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Open_WRONLY_Nested) {
  this->Test_Open_WRONLY(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Open_RDWR) {
  this->Test_Open_RDWR(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Open_RDWR_Nested) {
  this->Test_Open_RDWR(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange1) {
  this->Test_Truncate_DontChange1(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange1_Nested) {
  this->Test_Truncate_DontChange1(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_GrowTo1) {
  this->Test_Truncate_GrowTo1(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_GrowTo1_Nested) {
  this->Test_Truncate_GrowTo1(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_Grow) {
  this->Test_Truncate_Grow(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_Grow_Nested) {
  this->Test_Truncate_Grow(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange2) {
  this->Test_Truncate_DontChange2(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_DontChange2_Nested) {
  this->Test_Truncate_DontChange2(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_Shrink) {
  this->Test_Truncate_Shrink(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_Shrink_Nested) {
  this->Test_Truncate_Shrink(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_ShrinkTo0) {
  this->Test_Truncate_ShrinkTo0(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Truncate_ShrinkTo0_Nested) {
  this->Test_Truncate_ShrinkTo0(this->file_nested.get());
}

TYPED_TEST_P(FsppFileTest, Stat_CreatedFileIsEmpty) {
  this->Test_Stat_CreatedFileIsEmpty(this->file_root.get());
}

TYPED_TEST_P(FsppFileTest, Stat_CreatedFileIsEmpty_Nested) {
  this->Test_Stat_CreatedFileIsEmpty(this->file_nested.get());
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

//TODO stat
//TODO access
//TODO rename
//TODO utimens
//TODO unlink

#endif
