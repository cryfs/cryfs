#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_H_
#define MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_H_

#include <sys/fcntl.h>
#include <sys/stat.h>

#include "testutils/FileSystemTest.h"

template<class ConcreteFileSystemTestFixture>
class FsppSymlinkTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
};

TYPED_TEST_CASE_P(FsppSymlinkTest);

TYPED_TEST_P(FsppSymlinkTest, Create_AbsolutePath) {
  this->CreateSymlink("/mysymlink", "/my/symlink/target");
}

TYPED_TEST_P(FsppSymlinkTest, Create_RelativePath) {
  this->CreateSymlink("/mysymlink", "../target");
}

TYPED_TEST_P(FsppSymlinkTest, Read_AbsolutePath) {
  this->CreateSymlink("/mysymlink", "/my/symlink/target");
  EXPECT_EQ("/my/symlink/target", this->LoadSymlink("/mysymlink")->target());
}

TYPED_TEST_P(FsppSymlinkTest, Read_RelativePath) {
  this->CreateSymlink("/mysymlink", "../target");
  EXPECT_EQ("../target", this->LoadSymlink("/mysymlink")->target());
}

TYPED_TEST_P(FsppSymlinkTest, Remove) {
  this->CreateSymlink("/mysymlink", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mysymlink"));
  this->LoadSymlink("/mysymlink")->remove();
  EXPECT_EQ(boost::none, this->device->Load("/mysymlink"));
}

TYPED_TEST_P(FsppSymlinkTest, Remove_Nested) {
  this->CreateDir("/mytestdir");
  this->CreateSymlink("/mytestdir/mysymlink", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/mysymlink"));
  this->LoadSymlink("/mytestdir/mysymlink")->remove();
  EXPECT_EQ(boost::none, this->device->Load("/mytestdir/mysymlink"));
}

REGISTER_TYPED_TEST_CASE_P(FsppSymlinkTest,
  Create_AbsolutePath,
  Create_RelativePath,
  Read_AbsolutePath,
  Read_RelativePath,
  Remove,
  Remove_Nested
);

//TODO Other tests?
//TODO Test all operations do (or don't) affect timestamps correctly

#endif
