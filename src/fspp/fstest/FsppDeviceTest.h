#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDEVICETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPDEVICETEST_H_

#include "fspp/fs_interface/FuseErrnoException.h"

template<class ConcreteFileSystemTestFixture>
class FsppDeviceTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  void InitDirStructure() {
    this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/")->createSymlink("mysymlink", "/symlink/target", fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/")->createDir("myemptydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createAndOpenFile("myfile2", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createSymlink("mysymlink", "/symlink/target", fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createDir("mysubdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir/mysubdir")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir/mysubdir")->createSymlink("mysymlink", "/symlink/target", fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir/mysubdir")->createDir("mysubsubdir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
  }
};

//Unfortunately, googletest only allows 50 test cases per REGISTER_TYPED_TEST_SUITE_P, so we have to split it.
template<class ConcreteFileSystemTestFixture> class FsppDeviceTest_One: public FsppDeviceTest<ConcreteFileSystemTestFixture> {};
template<class ConcreteFileSystemTestFixture> class FsppDeviceTest_Two: public FsppDeviceTest<ConcreteFileSystemTestFixture> {};

TYPED_TEST_SUITE_P(FsppDeviceTest_One);
TYPED_TEST_SUITE_P(FsppDeviceTest_Two);

TYPED_TEST_P(FsppDeviceTest_One, InitFilesystem) {
  //fixture->createDevice() is called in the FileSystemTest constructor
}

TYPED_TEST_P(FsppDeviceTest_One, LoadRootDir_Load) {
  auto node = this->Load("/");
  this->EXPECT_IS_DIR(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadRootDir_LoadDir) {
  this->LoadDir("/");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadRootDir_LoadFile) {
  EXPECT_THROW(
    this->LoadFile("/"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadRootDir_LoadSymlink) {
  EXPECT_THROW(
    this->LoadSymlink("/"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromRootDir_Load) {
  this->InitDirStructure();
  auto node = this->Load("/myfile");
  this->EXPECT_IS_FILE(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromRootDir_LoadFile) {
  this->InitDirStructure();
  this->LoadFile("/myfile");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromRootDir_LoadDir) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadDir("/myfile"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromRootDir_LoadSymlink) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadSymlink("/myfile"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromRootDir_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir");
  this->EXPECT_IS_DIR(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromRootDir_LoadDir) {
  this->InitDirStructure();
  this->LoadDir("/mydir");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromRootDir_LoadFile) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadFile("/mydir"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromRootDir_LoadSymlink) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadSymlink("/mydir"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromRootDir_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mysymlink");
  this->EXPECT_IS_SYMLINK(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromRootDir_LoadSymlink) {
  this->InitDirStructure();
  this->LoadSymlink("/mysymlink");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromRootDir_LoadFile) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadFile("/mysymlink"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromRootDir_LoadDir) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadDir("/mysymlink"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromEmptyRootDir_Load) {
  EXPECT_EQ(boost::none, this->device->Load("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromEmptyRootDir_LoadDir) {
    EXPECT_EQ(boost::none, this->device->LoadDir("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromEmptyRootDir_LoadFile) {
    EXPECT_EQ(boost::none, this->device->LoadFile("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromEmptyRootDir_LoadSymlink) {
    EXPECT_EQ(boost::none, this->device->LoadSymlink("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromRootDir_Load) {
  this->InitDirStructure();
  EXPECT_EQ(boost::none, this->device->Load("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromRootDir_LoadDir) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadDir("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromRootDir_LoadFile) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadFile("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromRootDir_LoadSymlink) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadSymlink("/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromNonexistingDir_Load) {
  this->InitDirStructure();
  //TODO Change as soon as we have a concept of how to handle filesystem errors in the interface
  EXPECT_ANY_THROW(
      this->device->Load("/nonexisting/nonexisting2")
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromNonexistingDir_LoadDir) {
    this->InitDirStructure();
    //TODO Change as soon as we have a concept of how to handle filesystem errors in the interface
    EXPECT_ANY_THROW(
        this->device->LoadDir("/nonexisting/nonexisting2")
    );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromNonexistingDir_LoadFile) {
    this->InitDirStructure();
    //TODO Change as soon as we have a concept of how to handle filesystem errors in the interface
    EXPECT_ANY_THROW(
        this->device->LoadFile("/nonexisting/nonexisting2")
    );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromNonexistingDir_LoadSymlink) {
    this->InitDirStructure();
    //TODO Change as soon as we have a concept of how to handle filesystem errors in the interface
    EXPECT_ANY_THROW(
        this->device->LoadSymlink("/nonexisting/nonexisting2")
    );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingDir_Load) {
  this->InitDirStructure();
  EXPECT_EQ(boost::none, this->device->Load("/mydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingDir_LoadDir) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadDir("/mydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingDir_LoadFile) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadFile("/mydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingDir_LoadSymlink) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadSymlink("/mydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingEmptyDir_Load) {
  this->InitDirStructure();
  EXPECT_EQ(boost::none, this->device->Load("/myemptydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingEmptyDir_LoadDir) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadDir("/myemptydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingEmptyDir_LoadFile) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadFile("/myemptydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadNonexistingFromExistingEmptyDir_LoadSymlink) {
    this->InitDirStructure();
    EXPECT_EQ(boost::none, this->device->LoadSymlink("/myemptydir/nonexisting"));
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromDir_Nesting1_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir/myfile");
  this->EXPECT_IS_FILE(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromDir_Nesting1_LoadFile) {
  this->InitDirStructure();
  this->LoadFile("/mydir/myfile");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromDir_Nesting1_LoadDir) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadDir("/mydir/myfile"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadFileFromDir_Nesting1_LoadSymlink) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadSymlink("/mydir/myfile"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromDir_Nesting1_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir/mysubdir");
  this->EXPECT_IS_DIR(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromDir_Nesting1_LoadDir) {
  this->InitDirStructure();
  this->LoadDir("/mydir/mysubdir");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromDir_Nesting1_LoadFile) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadFile("/mydir/mysubdir"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadDirFromDir_Nesting1_LoadSymlink) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadSymlink("/mydir/mysubdir"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromDir_Nesting1_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir/mysymlink");
  this->EXPECT_IS_SYMLINK(node);
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromDir_Nesting1_LoadSymlink) {
  this->InitDirStructure();
  this->LoadSymlink("/mydir/mysymlink");
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromDir_Nesting1_LoadFile) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadFile("/mydir/mysymlink"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_One, LoadSymlinkFromDir_Nesting1_LoadDir) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadDir("/mydir/mysymlink"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadFileFromDir_Nesting2_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir/mysubdir/myfile");
  this->EXPECT_IS_FILE(node);
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadFileFromDir_Nesting2_LoadFile) {
  this->InitDirStructure();
  this->LoadFile("/mydir/mysubdir/myfile");
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadFileFromDir_Nesting2_LoadDir) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadDir("/mydir/mysubdir/myfile"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadFileFromDir_Nesting2_LoadSymlink) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadSymlink("/mydir/mysubdir/myfile"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadDirFromDir_Nesting2_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir/mysubdir/mysubsubdir");
  this->EXPECT_IS_DIR(node);
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadDirFromDir_Nesting2_LoadDir) {
  this->InitDirStructure();
  this->LoadDir("/mydir/mysubdir/mysubsubdir");
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadDirFromDir_Nesting2_LoadFile) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadFile("/mydir/mysubdir/mysubsubdir"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadDirFromDir_Nesting2_LoadSymlink) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadSymlink("/mydir/mysubdir/mysubsubdir"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadSymlinkFromDir_Nesting2_Load) {
  this->InitDirStructure();
  auto node = this->Load("/mydir/mysubdir/mysymlink");
  this->EXPECT_IS_SYMLINK(node);
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadSymlinkFromDir_Nesting2_LoadSymlink) {
  this->InitDirStructure();
  this->LoadSymlink("/mydir/mysubdir/mysymlink");
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadSymlinkFromDir_Nesting2_LoadFile) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadFile("/mydir/mysubdir/mysymlink"),
    fspp::fuse::FuseErrnoException
  );
}

TYPED_TEST_P(FsppDeviceTest_Two, LoadSymlinkFromDir_Nesting2_LoadDir) {
  this->InitDirStructure();
  EXPECT_THROW(
    this->LoadDir("/mydir/mysubdir/mysymlink"),
    fspp::fuse::FuseErrnoException
  );
}


//TODO Test statfs

REGISTER_TYPED_TEST_SUITE_P(FsppDeviceTest_One,
  InitFilesystem,
  LoadRootDir_Load,
  LoadRootDir_LoadDir,
  LoadRootDir_LoadFile,
  LoadRootDir_LoadSymlink,
  LoadFileFromRootDir_Load,
  LoadFileFromRootDir_LoadFile,
  LoadFileFromRootDir_LoadDir,
  LoadFileFromRootDir_LoadSymlink,
  LoadDirFromRootDir_Load,
  LoadDirFromRootDir_LoadDir,
  LoadDirFromRootDir_LoadFile,
  LoadDirFromRootDir_LoadSymlink,
  LoadSymlinkFromRootDir_Load,
  LoadSymlinkFromRootDir_LoadSymlink,
  LoadSymlinkFromRootDir_LoadFile,
  LoadSymlinkFromRootDir_LoadDir,
  LoadNonexistingFromEmptyRootDir_Load,
  LoadNonexistingFromEmptyRootDir_LoadDir,
  LoadNonexistingFromEmptyRootDir_LoadFile,
  LoadNonexistingFromEmptyRootDir_LoadSymlink,
  LoadNonexistingFromRootDir_Load,
  LoadNonexistingFromRootDir_LoadDir,
  LoadNonexistingFromRootDir_LoadFile,
  LoadNonexistingFromRootDir_LoadSymlink,
  LoadNonexistingFromNonexistingDir_Load,
  LoadNonexistingFromNonexistingDir_LoadDir,
  LoadNonexistingFromNonexistingDir_LoadFile,
  LoadNonexistingFromNonexistingDir_LoadSymlink,
  LoadNonexistingFromExistingDir_Load,
  LoadNonexistingFromExistingDir_LoadDir,
  LoadNonexistingFromExistingDir_LoadFile,
  LoadNonexistingFromExistingDir_LoadSymlink,
  LoadNonexistingFromExistingEmptyDir_Load,
  LoadNonexistingFromExistingEmptyDir_LoadDir,
  LoadNonexistingFromExistingEmptyDir_LoadFile,
  LoadNonexistingFromExistingEmptyDir_LoadSymlink,
  LoadFileFromDir_Nesting1_Load,
  LoadFileFromDir_Nesting1_LoadFile,
  LoadFileFromDir_Nesting1_LoadDir,
  LoadFileFromDir_Nesting1_LoadSymlink,
  LoadDirFromDir_Nesting1_Load,
  LoadDirFromDir_Nesting1_LoadDir,
  LoadDirFromDir_Nesting1_LoadFile,
  LoadDirFromDir_Nesting1_LoadSymlink,
  LoadSymlinkFromDir_Nesting1_Load,
  LoadSymlinkFromDir_Nesting1_LoadSymlink,
  LoadSymlinkFromDir_Nesting1_LoadFile,
  LoadSymlinkFromDir_Nesting1_LoadDir
);

REGISTER_TYPED_TEST_SUITE_P(FsppDeviceTest_Two,
  LoadFileFromDir_Nesting2_Load,
  LoadFileFromDir_Nesting2_LoadFile,
  LoadFileFromDir_Nesting2_LoadDir,
  LoadFileFromDir_Nesting2_LoadSymlink,
  LoadDirFromDir_Nesting2_Load,
  LoadDirFromDir_Nesting2_LoadDir,
  LoadDirFromDir_Nesting2_LoadFile,
  LoadDirFromDir_Nesting2_LoadSymlink,
  LoadSymlinkFromDir_Nesting2_Load,
  LoadSymlinkFromDir_Nesting2_LoadSymlink,
  LoadSymlinkFromDir_Nesting2_LoadFile,
  LoadSymlinkFromDir_Nesting2_LoadDir
);

//TODO Missing tests: LoadSymlink

#endif
