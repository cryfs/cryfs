#include <gtest/gtest.h>
#include "testutils/CryTestBase.h"
#include <cryfs/impl/filesystem/CryOpenFile.h>

using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using namespace cryfs;
namespace bf = boost::filesystem;

// Many generic (black box) test cases for FsppNode are covered in Fspp fstest.
// This class adds some tests that need insight into how CryFS works.

class CryNodeTest_Rename : public ::testing::Test, public CryTestBase {
};

TEST_F(CryNodeTest_Rename, DoesntLeaveBlocksOver) {
    auto node = CreateFile("/oldname");
    EXPECT_EQ(2u, device().numBlocks()); // In the beginning, there is two blocks (the root block and the created file). If that is not true anymore, we'll have to adapt the test case.
    node->rename("/newname");
    EXPECT_EQ(2u, device().numBlocks()); // Still same number of blocks
}

// TODO Add similar test cases (i.e. checking number of blocks) for other situations in rename, and also for other operations (e.g. deleting files).

TEST_F(CryNodeTest_Rename, Overwrite_DoesntLeaveBlocksOver) {
    auto node = CreateFile("/oldname");
    CreateFile("/newexistingname");
    EXPECT_EQ(3u, device().numBlocks()); // In the beginning, there is three blocks (the root block and the two created files). If that is not true anymore, we'll have to adapt the test case.
    node->rename("/newexistingname");
    EXPECT_EQ(2u, device().numBlocks()); // Only the blocks of one file are left
}

TEST_F(CryNodeTest_Rename, UpdatesParentPointers_File) {
    this->CreateDir("/mydir");
    auto node = this->CreateFile("/oldname");
    node->rename("/mydir/newname");
    EXPECT_TRUE(node->checkParentPointer());
}

TEST_F(CryNodeTest_Rename, UpdatesParentPointers_Dir) {
    this->CreateDir("/mydir");
    auto node = this->CreateDir("/oldname");
    node->rename("/mydir/newname");
    EXPECT_TRUE(node->checkParentPointer());
}

TEST_F(CryNodeTest_Rename, UpdatesParentPointers_Symlink) {
    this->CreateDir("/mydir");
    auto node = this->CreateSymlink("/oldname");
    node->rename("/mydir/newname");
    EXPECT_TRUE(node->checkParentPointer());
}
