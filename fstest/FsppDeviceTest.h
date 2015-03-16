#ifndef MESSMER_FSPP_FSTEST_FSPPDEVICETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPDEVICETEST_H_

template<class ConcreteFileSystemTestFixture>
class FsppDeviceTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
};

TYPED_TEST_CASE_P(FsppDeviceTest);

TYPED_TEST_P(FsppDeviceTest, InitFilesystem) {
  //fixture->createDevice() is called in the FileSystemTest constructor
}

TYPED_TEST_P(FsppDeviceTest, LoadRootDir) {
  this->LoadDir("/");
}

TYPED_TEST_P(FsppDeviceTest, LoadFileFromRootDir) {
  this->LoadDir("/")->createAndOpenFile("myfile", 0);
  this->LoadFile("/myfile");
}

TYPED_TEST_P(FsppDeviceTest, LoadDirFromRootDir) {
  this->LoadDir("/")->createDir("mydir", 0);
  this->LoadDir("/mydir");
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromRootDir) {
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/nonexisting")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromNonexistingDir) {
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/nonexisting/nonexisting2")
  );
}

TYPED_TEST_P(FsppDeviceTest, LoadNonexistingFromExistingDir) {
  this->LoadDir("/")->createDir("mydir", 0);
  //TODO Change, as soon as it's clear how we want to handle fs errors
  EXPECT_ANY_THROW(
    this->device->Load("/mydir/nonexisting")
  );
}

//TODO Load file/dir which is more nested
//TODO Load from dir structure with more than one entry per dir
//TODO statfs

REGISTER_TYPED_TEST_CASE_P(FsppDeviceTest,
  InitFilesystem,
  LoadRootDir,
  LoadFileFromRootDir,
  LoadDirFromRootDir,
  LoadNonexistingFromRootDir,
  LoadNonexistingFromNonexistingDir,
  LoadNonexistingFromExistingDir
);

#endif
