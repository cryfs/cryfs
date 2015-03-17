#ifndef MESSMER_FSPP_FSTEST_FSPPDEVICETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPDEVICETEST_H_

template<class ConcreteFileSystemTestFixture>
class FsppDeviceTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
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

TYPED_TEST_CASE_P(FsppDeviceTest);

TYPED_TEST_P(FsppDeviceTest, InitFilesystem) {
  //fixture->createDevice() is called in the FileSystemTest constructor
}

TYPED_TEST_P(FsppDeviceTest, LoadRootDir) {
  this->LoadDir("/");
}

TYPED_TEST_P(FsppDeviceTest, LoadFileFromRootDir) {
  this->InitDirStructure();
  this->LoadFile("/myfile");
}

TYPED_TEST_P(FsppDeviceTest, LoadDirFromRootDir) {
  this->InitDirStructure();
  this->LoadDir("/mydir");
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromEmptyRootDir) {
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/nonexisting")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromRootDir) {
  this->InitDirStructure();
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/nonexisting")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromNonexistingDir) {
  this->InitDirStructure();
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/nonexisting/nonexisting2")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromExistingDir) {
  this->InitDirStructure();
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/mydir/nonexisting")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromExistingEmptyDir) {
  this->InitDirStructure();
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/myemptydir/nonexisting")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadFileFromDir_Nesting1) {
  this->InitDirStructure();
  this->LoadFile("/mydir/myfile");
}

TYPED_TEST_P(FsppDeviceTest, LoadDirFromDir_Nesting1) {
  this->InitDirStructure();
  this->LoadDir("/mydir/mysubdir");
}

TYPED_TEST_P(FsppDeviceTest, LoadFileFromDir_Nesting2) {
  this->InitDirStructure();
  this->LoadFile("/mydir/mysubdir/myfile");
}

TYPED_TEST_P(FsppDeviceTest, LoadDirFromDir_Nesting2) {
  this->InitDirStructure();
  this->LoadDir("/mydir/mysubdir/mysubsubdir");
}

//TODO Test statfs

REGISTER_TYPED_TEST_CASE_P(FsppDeviceTest,
  InitFilesystem,
  LoadRootDir,
  LoadFileFromRootDir,
  LoadDirFromRootDir,
  LoadNonexistingFromEmptyRootDir,
  LoadNonexistingFromRootDir,
  LoadNonexistingFromNonexistingDir,
  LoadNonexistingFromExistingDir,
  LoadNonexistingFromExistingEmptyDir,
  LoadFileFromDir_Nesting1,
  LoadDirFromDir_Nesting1,
  LoadFileFromDir_Nesting2,
  LoadDirFromDir_Nesting2
);

#endif
