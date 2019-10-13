#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDIRTEST_H_
#define MESSMER_FSPP_FSTEST_FSPPDIRTEST_H_

template<class ConcreteFileSystemTestFixture>
class FsppDirTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  void InitDirStructure() {
    this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/")->createDir("myemptydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createAndOpenFile("myfile2", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir/mysubdir")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir/mysubdir")->createDir("mysubsubdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  }

  void EXPECT_CHILDREN_ARE(const boost::filesystem::path &path, const std::initializer_list<fspp::Dir::Entry> expected) {
    EXPECT_CHILDREN_ARE(this->LoadDir(path).get(), expected);
  }

  void EXPECT_CHILDREN_ARE(fspp::Dir *dir, const std::initializer_list<fspp::Dir::Entry> expected) {
	std::vector<fspp::Dir::Entry> expectedChildren = expected;
	expectedChildren.push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
	expectedChildren.push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
	EXPECT_UNORDERED_EQ(expectedChildren, dir->children());
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
TYPED_TEST_SUITE_P(FsppDirTest);

inline fspp::Dir::Entry DirEntry(const std::string &name) {
  return fspp::Dir::Entry(fspp::Dir::EntryType::DIR, name);
}

inline fspp::Dir::Entry FileEntry(const std::string &name) {
  return fspp::Dir::Entry(fspp::Dir::EntryType::FILE, name);
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_Empty) {
  this->EXPECT_CHILDREN_ARE("/", {});
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneFile_Directly) {
  auto rootdir = this->LoadDir("/");
  rootdir->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE(rootdir.get(), {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneFile_AfterReloadingDir) {
  this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/", {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneDir_Directly) {
  auto rootdir = this->LoadDir("/");
  rootdir->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE(rootdir.get(), {
    DirEntry("mydir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_RootDir_OneDir_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
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
  this->LoadDir("/")->createDir("myemptydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/myemptydir", {});
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  auto dir = this->LoadDir("/mydir");
  dir->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE(dir.get(), {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneFile_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/mydir", {
    FileEntry("myfile")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_Directly) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  auto dir = this->LoadDir("/mydir");
  dir->createDir("mysubdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE(dir.get(), {
    DirEntry("mysubdir")
  });
}

TYPED_TEST_P(FsppDirTest, Children_Nested_OneDir_AfterReloadingDir) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
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

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_InEmptyRoot) {
  this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->LoadFile("/myfile");
  this->Load("/myfile"); // Test that we can also load the file node
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_InNonemptyRoot) {
  this->InitDirStructure();
  this->LoadDir("/")->createAndOpenFile("mynewfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/", {
    FileEntry("myfile"),
	DirEntry("mydir"),
	DirEntry("myemptydir"),
	FileEntry("mynewfile")
  });
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_InEmptyNestedDir) {
  this->InitDirStructure();
  this->LoadDir("/myemptydir")->createAndOpenFile("mynewfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/myemptydir", {
	FileEntry("mynewfile")
  });
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_InNonemptyNestedDir) {
  this->InitDirStructure();
  this->LoadDir("/mydir")->createAndOpenFile("mynewfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/mydir", {
    FileEntry("myfile"),
	FileEntry("myfile2"),
	DirEntry("mysubdir"),
	FileEntry("mynewfile")
  });
}

TYPED_TEST_P(FsppDirTest, CreateAndOpenFile_AlreadyExisting) {
  this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  //TODO Change, once we know which way of error reporting we want for such errors
  EXPECT_ANY_THROW(
    this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  );
}

TYPED_TEST_P(FsppDirTest, CreateDir_InEmptyRoot) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->LoadDir("/mydir");
  this->Load("/mydir"); // Test we can also load the dir node
}

TYPED_TEST_P(FsppDirTest, CreateDir_InNonemptyRoot) {
  this->InitDirStructure();
  this->LoadDir("/")->createDir("mynewdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/", {
    FileEntry("myfile"),
	DirEntry("mydir"),
	DirEntry("myemptydir"),
	DirEntry("mynewdir")
  });
}

TYPED_TEST_P(FsppDirTest, CreateDir_InEmptyNestedDir) {
  this->InitDirStructure();
  this->LoadDir("/myemptydir")->createDir("mynewdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/myemptydir", {
	DirEntry("mynewdir")
  });
}

TYPED_TEST_P(FsppDirTest, CreateDir_InNonemptyNestedDir) {
  this->InitDirStructure();
  this->LoadDir("/mydir")->createDir("mynewdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  this->EXPECT_CHILDREN_ARE("/mydir", {
    FileEntry("myfile"),
	FileEntry("myfile2"),
	DirEntry("mysubdir"),
	DirEntry("mynewdir")
  });
}

TYPED_TEST_P(FsppDirTest, CreateDir_AlreadyExisting) {
  this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  //TODO Change, once we know which way of error reporting we want for such errors
  EXPECT_ANY_THROW(
    this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  );
}

TYPED_TEST_P(FsppDirTest, Remove) {
  this->CreateDir("/mytestdir");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir"));
  EXPECT_NE(boost::none, this->device->LoadDir("/mytestdir"));
  this->Load("/mytestdir")->remove();
  EXPECT_EQ(boost::none, this->device->Load("/mytestdir"));
  EXPECT_EQ(boost::none, this->device->LoadDir("/mytestdir"));
}

TYPED_TEST_P(FsppDirTest, Remove_Nested) {
  this->CreateDir("/mytestdir");
  this->CreateDir("/mytestdir/mydir");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/mydir"));
  EXPECT_NE(boost::none, this->device->LoadDir("/mytestdir/mydir"));
  this->Load("/mytestdir/mydir")->remove();
  EXPECT_EQ(boost::none, this->device->Load("/mytestdir/mydir"));
  EXPECT_EQ(boost::none, this->device->LoadDir("/mytestdir/mydir"));
}

REGISTER_TYPED_TEST_SUITE_P(FsppDirTest,
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
  CreateAndOpenFile_InEmptyRoot,
  CreateAndOpenFile_InNonemptyRoot,
  CreateAndOpenFile_InEmptyNestedDir,
  CreateAndOpenFile_InNonemptyNestedDir,
  CreateAndOpenFile_AlreadyExisting,
  CreateDir_InEmptyRoot,
  CreateDir_InNonemptyRoot,
  CreateDir_InEmptyNestedDir,
  CreateDir_InNonemptyNestedDir,
  CreateDir_AlreadyExisting,
  Remove,
  Remove_Nested
);


//TODO rmdir (also test that deleting a non-empty dir returns ENOTEMPTY, because otherwise there might not be any unlink syscalls for the entries issued)
//TODO mkdir with uid/gid
//TODO createAndOpenFile: all stat values correctly set (1. in the OpenFile instance returned from createAndOpenFile and 2. on an lstat on the file object afterwards)
//TODO Test all operations do (or don't) affect dir timestamps correctly

#endif
