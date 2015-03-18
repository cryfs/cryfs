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
};
TYPED_TEST_CASE_P(FsppDirTest);

TYPED_TEST_P(FsppDirTest, Children_RootDir_Empty) {
  auto children = this->LoadDir("/")->children();
  EXPECT_EQ(0, children->size());
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneFile) {
  auto rootdir = this->LoadDir("/");
  rootdir->createAndOpenFile("myfile", this->MODE_PUBLIC);
  auto children = rootdir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneDir) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("mydir", this->MODE_PUBLIC);
  auto children = rootdir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[0].type);
  EXPECT_EQ("mydir", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_LargerStructure) {
  this->InitDirStructure();
  auto children = this->LoadDir("/")->children();
  EXPECT_EQ(3, children->size());
  //TODO Ignore order
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[1].type);
  EXPECT_EQ("mydir", (*children)[1].name);
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[2].type);
  EXPECT_EQ("myemptydir", (*children)[2].name);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_Empty) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("myemptydir", this->MODE_PUBLIC);
  auto children = this->LoadDir("/myemptydir")->children();
  EXPECT_EQ(0, children->size());
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  dir->createAndOpenFile("myfile", this->MODE_PUBLIC);
  auto children = dir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  auto children = dir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  dir->createDir("mysubdir", this->MODE_PUBLIC);
  auto children = dir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[0].type);
  EXPECT_EQ("mysubdir", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  auto children = dir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[0].type);
  EXPECT_EQ("mysubdir", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_Nested_LargerStructure_Empty) {
  this->InitDirStructure();
  auto children = this->LoadDir("/myemptydir")->children();
  EXPECT_EQ(0, children->size());
}

TYPED_TEST_P(FsppDirTest, Children_Nested_LargerStructure) {
  this->InitDirStructure();
  auto children = this->LoadDir("/mydir")->children();
  EXPECT_EQ(3, children->size());
  //TODO Ignore order
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[1].type);
  EXPECT_EQ("myfile2", (*children)[1].name);
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[2].type);
  EXPECT_EQ("mysubdir", (*children)[2].name);
}

TYPED_TEST_P(FsppDirTest, Children_Nested2_LargerStructure) {
  this->InitDirStructure();
  auto children = this->LoadDir("/mydir/mysubdir")->children();
  EXPECT_EQ(2, children->size());
  //TODO Ignore order
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[1].type);
  EXPECT_EQ("mysubsubdir", (*children)[1].name);
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

//TODO rmdir

#endif
