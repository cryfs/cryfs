#ifndef MESSMER_FSPP_FSTEST_FSPPDIRTEST_H_
#define MESSMER_FSPP_FSTEST_FSPPDIRTEST_H_

template<class ConcreteFileSystemTestFixture>
class FsppDirTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
};
TYPED_TEST_CASE_P(FsppDirTest);

TYPED_TEST_P(FsppDirTest, Children_EmptyRootDir) {
  auto rootdir = this->LoadDir("/");
  auto children = rootdir->children();
  EXPECT_EQ(0, children->size());
}

TYPED_TEST_P(FsppDirTest, Children_OneFileInRootDir) {
  auto rootdir = this->LoadDir("/");
  rootdir->createAndOpenFile("myfile", 0);
  auto children = rootdir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::FILE, (*children)[0].type);
  EXPECT_EQ("myfile", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, Children_OneDirInRootDir) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("mydir", 0);
  auto children = rootdir->children();
  EXPECT_EQ(1, children->size());
  EXPECT_EQ(fspp::Dir::EntryType::DIR, (*children)[0].type);
  EXPECT_EQ("mydir", (*children)[0].name);
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_LoadAfterwards) {
  this->LoadDir("/")->createAndOpenFile("myfile", 0);
  this->LoadFile("/myfile");
}

TYPED_TEST_P(FsppDirTest, CreateDir_LoadAfterwards) {
  this->LoadDir("/")->createDir("mydir", 0);
  this->LoadDir("/mydir");
}

REGISTER_TYPED_TEST_CASE_P(FsppDirTest,
  Children_EmptyRootDir,
  Children_OneFileInRootDir,
  Children_OneDirInRootDir,
  CreateAndOpenFile_LoadAfterwards,
  CreateDir_LoadAfterwards
);

//TODO Add File/Dir to subdir/subsubdir and check entries
//TODO Build dir structure with more than one entry

//TODO rmdir

#endif
