#ifndef MESSMER_FSPP_FSTEST_FSPPDIRTEST_H_
#define MESSMER_FSPP_FSTEST_FSPPDIRTEST_H_

template<class ConcreteFileSystemTestFixture>
class FsppDirTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  void InitDirStructure() {
    this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
    this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
    this->LoadDir("/")->createDir("myemptydir", this->MODE_PUBLIC);
    this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC);
    this->LoadDir("/mydir")->createAndOpenFile("myfile2", this->MODE_PUBLIC);
    this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC);
    this->LoadDir("/mydir/mysubdir")->createAndOpenFile("myfile", this->MODE_PUBLIC);
    this->LoadDir("/mydir/mysubdir")->createDir("mysubsubdir", this->MODE_PUBLIC);
  }

  void EXPECT_HAS_DEFAULT_ENTRIES(const std::vector<fspp::Dir::Entry> &children) {
	EXPECT_HAS_DIR_ENTRY(".", children);
	EXPECT_HAS_DIR_ENTRY("..", children);
  }

  void EXPECT_HAS_DIR_ENTRY(const std::string &name, const std::vector<fspp::Dir::Entry> &children) {
	EXPECT_HAS_ENTRY(fspp::Dir::EntryType::DIR, name, children);
  }

  void EXPECT_HAS_FILE_ENTRY(const std::string &name, const std::vector<fspp::Dir::Entry> &children) {
	EXPECT_HAS_ENTRY(fspp::Dir::EntryType::FILE, name, children);
  }

  void EXPECT_HAS_ENTRY(fspp::Dir::EntryType type, const std::string &name, const std::vector<fspp::Dir::Entry> &children) {
	for (const auto &child : children) {
	  if (child.type == type && child.name == name) {
        return;
	  }
	}
	EXPECT_TRUE(false);
  }
};
TYPED_TEST_CASE_P(FsppDirTest);

TYPED_TEST_P(FsppDirTest, Children_RootDir_Empty) {
  auto children = this->LoadDir("/")->children();
  EXPECT_EQ(2, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneFile) {
  auto rootdir = this->LoadDir("/");
  rootdir->createAndOpenFile("myfile", this->MODE_PUBLIC);
  auto children = rootdir->children();
  EXPECT_EQ(3, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_FILE_ENTRY("myfile", *children);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneDir) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("mydir", this->MODE_PUBLIC);
  auto children = rootdir->children();
  EXPECT_EQ(3, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_DIR_ENTRY("mydir", *children);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_LargerStructure) {
  this->InitDirStructure();
  auto children = this->LoadDir("/")->children();
  EXPECT_EQ(5, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_FILE_ENTRY("myfile", *children);
  this->EXPECT_HAS_DIR_ENTRY("mydir", *children);
  this->EXPECT_HAS_DIR_ENTRY("myemptydir", *children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_Empty) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("myemptydir", this->MODE_PUBLIC);
  auto children = this->LoadDir("/myemptydir")->children();
  EXPECT_EQ(2, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  dir->createAndOpenFile("myfile", this->MODE_PUBLIC);
  auto children = dir->children();
  EXPECT_EQ(3, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_FILE_ENTRY("myfile", *children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  auto children = dir->children();
  EXPECT_EQ(3, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_FILE_ENTRY("myfile", *children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  dir->createDir("mysubdir", this->MODE_PUBLIC);
  auto children = dir->children();
  EXPECT_EQ(3, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_DIR_ENTRY("mysubdir", *children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  auto children = dir->children();
  EXPECT_EQ(3, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_DIR_ENTRY("mysubdir", *children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_LargerStructure_Empty) {
  this->InitDirStructure();
  auto children = this->LoadDir("/myemptydir")->children();
  EXPECT_EQ(2, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_LargerStructure) {
  this->InitDirStructure();
  auto children = this->LoadDir("/mydir")->children();
  EXPECT_EQ(5, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_FILE_ENTRY("myfile", *children);
  this->EXPECT_HAS_FILE_ENTRY("myfile2", *children);
  this->EXPECT_HAS_DIR_ENTRY("mysubdir", *children);
}

TYPED_TEST_P(FsppDirTest, Children_Nested2_LargerStructure) {
  this->InitDirStructure();
  auto children = this->LoadDir("/mydir/mysubdir")->children();
  EXPECT_EQ(4, children->size());
  this->EXPECT_HAS_DEFAULT_ENTRIES(*children);
  this->EXPECT_HAS_FILE_ENTRY("myfile", *children);
  this->EXPECT_HAS_DIR_ENTRY("mysubsubdir", *children);
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_LoadAfterwards) {
  this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  this->LoadFile("/myfile");
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_AlreadyExisting) {
  this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  //TODO Change, once we know which way of error reporting we want for such errors
  EXPECT_ANY_THROW(
    this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  );
}

TYPED_TEST_P(FsppDirTest, CreateDir_LoadAfterwards) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir");
}

TYPED_TEST_P(FsppDirTest, CreateDir_AlreadyExisting) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  //TODO Change, once we know which way of error reporting we want for such errors
  EXPECT_ANY_THROW(
    this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  );
}

REGISTER_TYPED_TEST_CASE_P(FsppDirTest,
  Children_RootDir_Empty,
  Children_RootDir_OneFile,
  Children_RootDir_OneDir,
  Children_RootDir_LargerStructure,
  Children_Nested_Empty,
  Children_Nested_OneFile_Directly,
  Children_Nested_OneFile_AfterReloadingDir,
  Children_Nested_OneDir_Directly,
  Children_Nested_OneDir_AfterReloadingDir,
  Children_Nested_LargerStructure,
  Children_Nested_LargerStructure_Empty,
  Children_Nested2_LargerStructure,
  CreateAndOpenFile_LoadAfterwards,
  CreateAndOpenFile_AlreadyExisting,
  CreateDir_LoadAfterwards,
  CreateDir_AlreadyExisting
);

//TODO stat
//TODO access
//TODO rename
//TODO utimens
//TODO rmdir

#endif
