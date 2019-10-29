#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_H_
#define MESSMER_FSPP_FSTEST_FSPPSYMLINKTEST_H_

#include <sys/stat.h>

#include "testutils/FileSystemTest.h"

template<class ConcreteFileSystemTestFixture>
class FsppSymlinkTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
};

TYPED_TEST_SUITE_P(FsppSymlinkTest);

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

TYPED_TEST_P(FsppSymlinkTest, Remove_Only_Node) {
  this->CreateSymlink("/mysymlink", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mysymlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mysymlink"));
  auto node = this->Load("/mysymlink");
  auto id = node->blockId();
  node->remove();
  EXPECT_TRUE(this->IsSymlinkInDir("/mysymlink"));
  EXPECT_FALSE(this->BlobExists(id));
}

TYPED_TEST_P(FsppSymlinkTest, Remove_Properly) {
  this->CreateSymlink("/mysymlink", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mysymlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mysymlink"));
  auto id = this->Load("/mysymlink")->blockId();
  this->filesystem.unlink("/mysymlink");
  EXPECT_FALSE(this->IsSymlinkInDir("/mysymlink"));
  EXPECT_FALSE(this->BlobExists(id));
  EXPECT_EQ(boost::none, this->device->Load("/mysymlink"));
  EXPECT_EQ(boost::none, this->device->LoadSymlink("/mysymlink"));
}

TYPED_TEST_P(FsppSymlinkTest, Remove_Nested) {
  this->CreateDir("/mytestdir");
  this->CreateSymlink("/mytestdir/mysymlink", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/mysymlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestdir/mysymlink"));
  this->filesystem.unlink("/mytestdir/mysymlink");
  EXPECT_EQ(boost::none, this->device->Load("/mytestdir/mysymlink"));
  EXPECT_EQ(boost::none, this->device->LoadSymlink("/mytestdir/mysymlink"));
}

TYPED_TEST_P(FsppSymlinkTest, Link_Root_Directory) {
  this->CreateSymlink("/mytestfile", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mytestfile"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestfile"));
  this->EXCPECT_NLINKS("/mytestfile", 1);
  this->filesystem.link("/mytestfile", "/myhardlink");
  EXPECT_NE(boost::none, this->device->Load("/myhardlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/myhardlink"));
  this->EXCPECT_NLINKS("/mytestfile", 2);
  this->EXCPECT_NLINKS("/myhardlink", 2);
  auto node = this->Load("/mytestfile");
  auto node2 = this->Load("/myhardlink");
  auto id = node->blockId();
  EXPECT_EQ(id, node2->blockId());
  this->filesystem.unlink("/mytestfile");
  EXPECT_FALSE(this->IsSymlinkInDir("/mytestfile"));
  EXPECT_TRUE(this->IsSymlinkInDir("/myhardlink"));
  EXPECT_TRUE(this->BlobExists(id));
  EXPECT_NE(boost::none, this->device->Load("/myhardlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/myhardlink"));
  this->EXCPECT_NLINKS("/myhardlink", 1);
  this->filesystem.unlink("/myhardlink");
  EXPECT_FALSE(this->IsSymlinkInDir("/myhardlink"));
  EXPECT_FALSE(this->BlobExists(id));
}

TYPED_TEST_P(FsppSymlinkTest, Link_Nested) {
  this->CreateDir("/mytestdir");
  this->CreateSymlink("/mytestdir/mytestfile", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/mytestfile"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestdir/mytestfile"));
  this->EXCPECT_NLINKS("/mytestdir/mytestfile", 1);
  this->filesystem.link("/mytestdir/mytestfile", "/mytestdir/myhardlink");
  this->EXCPECT_NLINKS("/mytestdir/mytestfile", 2);
  this->EXCPECT_NLINKS("/mytestdir/myhardlink", 2);
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/myhardlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestdir/myhardlink"));
  auto node = this->Load("/mytestdir/mytestfile");
  auto node2 = this->Load("/mytestdir/myhardlink");
  auto id = node->blockId();
  EXPECT_EQ(id, node2->blockId());
  this->filesystem.unlink("/mytestdir/mytestfile");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/myhardlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestdir/myhardlink"));
  EXPECT_FALSE(this->IsSymlinkInDir("/mytestdir/mytestfile"));
  EXPECT_TRUE(this->IsSymlinkInDir("/mytestdir/myhardlink"));
  EXPECT_TRUE(this->BlobExists(id));
  this->EXCPECT_NLINKS("/mytestdir/myhardlink", 1);
  this->filesystem.unlink("/mytestdir/myhardlink");
  EXPECT_FALSE(this->IsSymlinkInDir("/mytestdir/myhardlink"));
  EXPECT_FALSE(this->BlobExists(id));
}

TYPED_TEST_P(FsppSymlinkTest, Link_Down) {
  this->CreateDir("/mytestdir");
  this->CreateSymlink("/mytestfile", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mytestfile"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestfile"));
  this->EXCPECT_NLINKS("/mytestfile", 1);
  this->filesystem.link("/mytestfile", "/mytestdir/myhardlink");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/myhardlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestdir/myhardlink"));
  this->EXCPECT_NLINKS("/mytestfile", 2);
  this->EXCPECT_NLINKS("/mytestdir/myhardlink", 2);
  auto node = this->Load("/mytestfile");
  auto node2 = this->Load("/mytestdir/myhardlink");
  auto id = node->blockId();
  EXPECT_EQ(id, node2->blockId());
  this->filesystem.unlink("/mytestfile");
  EXPECT_FALSE(this->IsSymlinkInDir("/mytestfile"));
  EXPECT_TRUE(this->IsSymlinkInDir("/mytestdir/myhardlink"));
  EXPECT_TRUE(this->BlobExists(id));
  this->EXCPECT_NLINKS("/mytestdir/myhardlink", 1);
  this->filesystem.unlink("/mytestdir/myhardlink");
  EXPECT_FALSE(this->IsSymlinkInDir("/mytestdir/myhardlink"));
  EXPECT_FALSE(this->BlobExists(id));
}

TYPED_TEST_P(FsppSymlinkTest, Link_Up) {
  this->CreateDir("/mytestdir");
  this->CreateSymlink("/mytestdir/mytestfile", "/my/symlink/target");
  EXPECT_NE(boost::none, this->device->Load("/mytestdir/mytestfile"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/mytestdir/mytestfile"));
  this->EXCPECT_NLINKS("/mytestdir/mytestfile", 1);
  this->filesystem.link("/mytestdir/mytestfile", "/myhardlink");
  EXPECT_NE(boost::none, this->device->Load("/myhardlink"));
  EXPECT_NE(boost::none, this->device->LoadSymlink("/myhardlink"));
  this->EXCPECT_NLINKS("/mytestdir/mytestfile", 2);
  this->EXCPECT_NLINKS("/myhardlink", 2);
  auto node = this->Load("/mytestdir/mytestfile");
  auto node2 = this->Load("/myhardlink");
  auto id = node->blockId();
  EXPECT_EQ(id, node2->blockId());
  this->filesystem.unlink("/mytestdir/mytestfile");
  EXPECT_FALSE(this->IsSymlinkInDir("/mytestdir/mytestfile"));
  EXPECT_TRUE(this->IsSymlinkInDir("/myhardlink"));
  EXPECT_TRUE(this->BlobExists(id));
  this->EXCPECT_NLINKS("/myhardlink", 1);
  this->filesystem.unlink("/myhardlink");
  EXPECT_FALSE(this->IsSymlinkInDir("/myhardlink"));
  EXPECT_FALSE(this->BlobExists(id));
}

REGISTER_TYPED_TEST_SUITE_P(FsppSymlinkTest,
  Create_AbsolutePath,
  Create_RelativePath,
  Read_AbsolutePath,
  Read_RelativePath,
  Remove_Only_Node,
  Remove_Properly,
  Remove_Nested,
  Link_Root_Directory,
  Link_Up,
  Link_Down,
  Link_Nested
);

//TODO Other tests?
//TODO Test all operations do (or don't) affect timestamps correctly

#endif
