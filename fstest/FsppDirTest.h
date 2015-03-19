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

  void EXPECT_CHILDREN_ARE(const boost::filesystem::path &path, const std::initializer_list<fspp::Dir::Entry> expected) {
	EXPECT_CHILDREN_ARE(*this->LoadDir(path), expected);
  }

  void EXPECT_CHILDREN_ARE(const fspp::Dir &dir, const std::initializer_list<fspp::Dir::Entry> expected) {
	std::vector<fspp::Dir::Entry> expectedChildren = expected;
	expectedChildren.push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
	expectedChildren.push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
	EXPECT_UNORDERED_EQ(expectedChildren, *dir.children());
  }

  template<class Entry>
  void EXPECT_UNORDERED_EQ(const std::vector<Entry> &expected, std::vector<Entry> actual) {
	EXPECT_EQ(expected.size(), actual.size());
	for (const Entry &expectedEntry : expected) {
	  removeOne(&actual, expectedEntry);
	}
  }

  template<class Entry>
  void removeOne(std::vector<Entry> *entries, const Entry &toRemove) {
	for (auto iter = entries->begin(); iter != entries->end(); ++iter) {
	  if (iter->type == toRemove.type && iter->name == toRemove.name) {
		entries->erase(iter);
		return;
	  }
	}
	EXPECT_TRUE(false);
  }
};
TYPED_TEST_CASE_P(FsppDirTest);

fspp::Dir::Entry DirEntry(const std::string &name) {
  return fspp::Dir::Entry(fspp::Dir::EntryType::DIR, name);
}

fspp::Dir::Entry FileEntry(const std::string &name) {
  return fspp::Dir::Entry(fspp::Dir::EntryType::FILE, name);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_Empty) {
  this->EXPECT_CHILDREN_ARE("/", {});
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneFile_Directly) {
  auto rootdir = this->LoadDir("/");
  rootdir->createAndOpenFile("myfile", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE(*rootdir, {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneFile_AfterReloadingDir) {
  this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE("/", {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneDir_Directly) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("mydir", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE(*rootdir, {
    DirEntry("mydir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneDir_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE("/", {
    DirEntry("mydir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_LargerStructure) {
  this->InitDirStructure();
  this->EXPECT_CHILDREN_ARE("/", {
    FileEntry("myfile"),
	DirEntry("mydir"),
	DirEntry("myemptydir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_Empty) {
  this->LoadDir("/")->createDir("myemptydir", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE("/myemptydir", {});
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  dir->createAndOpenFile("myfile", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE(*dir, {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE("/mydir", {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  auto dir = this->LoadDir("/mydir");
  dir->createDir("mysubdir", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE(*dir, {
    DirEntry("mysubdir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC);
  this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC);
  this->EXPECT_CHILDREN_ARE("/mydir", {
    DirEntry("mysubdir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_LargerStructure_Empty) {
  this->InitDirStructure();
  this->EXPECT_CHILDREN_ARE("/myemptydir", {});
}

TYPED_TEST_P(FsppDirTest, Children_Nested_LargerStructure) {
  this->InitDirStructure();
  this->EXPECT_CHILDREN_ARE("/mydir", {
    FileEntry("myfile"),
	FileEntry("myfile2"),
	DirEntry("mysubdir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested2_LargerStructure) {
  this->InitDirStructure();
  this->EXPECT_CHILDREN_ARE("/mydir/mysubdir", {
	FileEntry("myfile"),
	DirEntry("mysubsubdir")
  });
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
  Children_RootDir_OneFile_Directly,
  Children_RootDir_OneFile_AfterReloadingDir,
  Children_RootDir_OneDir_Directly,
  Children_RootDir_OneDir_AfterReloadingDir,
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
