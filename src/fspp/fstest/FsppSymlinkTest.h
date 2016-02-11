#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_H_
#define MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_H_

#include <sys/fcntl.h>
#include <sys/stat.h>

#include "testutils/FileSystemTest.h"

template<class ConcreteFileSystemTestFixture>
class FsppSymlinkTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  void CreateSymlink(const std::string &source, const boost::filesystem::path &target) {
    this->LoadDir("/")->createSymlink(source, target, 0, 0);
  }

  void Test_Create_AbsolutePath() {
    CreateSymlink("mysymlink", "/my/symlink/target");
  }

  void Test_Create_RelativePath() {
    CreateSymlink("mysymlink", "../target");
  }

  void Test_Read_AbsolutePath() {
    CreateSymlink("mysymlink", "/my/symlink/target");
    EXPECT_EQ("/my/symlink/target", this->LoadSymlink("/mysymlink")->target());
  }

  void Test_Read_RelativePath() {
    CreateSymlink("mysymlink", "../target");
    EXPECT_EQ("../target", this->LoadSymlink("/mysymlink")->target());
  }

  void Test_Delete() {
    CreateSymlink("mysymlink", "/my/symlink/target");
    std::cerr << "1" << std::endl;
    EXPECT_NE(boost::none, this->device->Load("/mysymlink"));
    std::cerr << "2" << std::endl;
    this->LoadSymlink("/mysymlink")->remove();
    std::cerr << "3" << std::endl;
    EXPECT_EQ(boost::none, this->device->Load("/mysymlink"));
  }
};

TYPED_TEST_CASE_P(FsppSymlinkTest);

TYPED_TEST_P(FsppSymlinkTest, Create_AbsolutePath) {
  this->Test_Create_AbsolutePath();
}

TYPED_TEST_P(FsppSymlinkTest, Create_RelativePath) {
  this->Test_Create_RelativePath();
}

TYPED_TEST_P(FsppSymlinkTest, Read_AbsolutePath) {
  this->Test_Read_AbsolutePath();
}

TYPED_TEST_P(FsppSymlinkTest, Read_RelativePath) {
  this->Test_Read_RelativePath();
}

TYPED_TEST_P(FsppSymlinkTest, Delete) {
  this->Test_Delete();
}

REGISTER_TYPED_TEST_CASE_P(FsppSymlinkTest,
  Create_AbsolutePath,
  Create_RelativePath,
  Read_AbsolutePath,
  Read_RelativePath,
  Delete
);

//TODO Other tests?

#endif
