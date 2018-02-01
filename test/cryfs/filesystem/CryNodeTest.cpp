#include <gtest/gtest.h>
#include "testutils/CryTestBase.h"
#include <cryfs/filesystem/CryDir.h>
#include <cryfs/filesystem/CryFile.h>
#include <cryfs/filesystem/CryOpenFile.h>

using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using namespace cryfs;
namespace bf = boost::filesystem;

// Many generic (black box) test cases for FsppNode are covered in Fspp fstest.
// This class adds some tests that need insight into how CryFS works.

class CryNodeTest : public ::testing::Test, public CryTestBase {
public:
    static constexpr mode_t MODE_PUBLIC = S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IWGRP | S_IXGRP | S_IROTH | S_IWOTH | S_IXOTH;

    unique_ref<CryNode> CreateFile(const bf::path &path) {
        auto parentDir = device().LoadDir(path.parent_path()).value();
        parentDir->createAndOpenFile(path.filename().native(), MODE_PUBLIC, 0, 0);
        //cpputils::destruct(std::move(parentDir));
        auto file = device().Load(path).value();
        return dynamic_pointer_move<CryNode>(file).value();
    }

    unique_ref<CryNode> CreateDir(const bf::path &path) {
        auto _parentDir = device().Load(path.parent_path()).value();
        auto parentDir = dynamic_pointer_move<CryDir>(_parentDir).value();
        parentDir->createDir(path.filename().native(), MODE_PUBLIC, 0, 0);
        //cpputils::destruct(std::move(parentDir));
        auto createdDir = device().Load(path).value();
        return dynamic_pointer_move<CryNode>(createdDir).value();
    }

    unique_ref<CryNode> CreateSymlink(const bf::path &path) {
        auto _parentDir = device().Load(path.parent_path()).value();
        auto parentDir = dynamic_pointer_move<CryDir>(_parentDir).value();
        parentDir->createSymlink(path.filename().native(), "/target", 0, 0);
        //cpputils::destruct(std::move(parentDir));
        auto createdSymlink = device().Load(path).value();
        return dynamic_pointer_move<CryNode>(createdSymlink).value();
    }
};

TEST_F(CryNodeTest, Rename_DoesntLeaveBlocksOver) {
    auto node = CreateFile("/oldname");
    EXPECT_EQ(2u, device().numBlocks()); // In the beginning, there is two blocks (the root block and the created file). If that is not true anymore, we'll have to adapt the test case.
    node->rename("/newname");
    EXPECT_EQ(2u, device().numBlocks()); // Still same number of blocks
}

// TODO Add similar test cases (i.e. checking number of blocks) for other situations in rename, and also for other operations (e.g. deleting files).

TEST_F(CryNodeTest, Rename_Overwrite_DoesntLeaveBlocksOver) {
    CreateFile("/oldname");
    CreateFile("/newexistingname");
    EXPECT_EQ(3u, device().numBlocks()); // In the beginning, there is three blocks (the root block and the two created files). If that is not true anymore, we'll have to adapt the test case.
    device().Load("/oldname").value()->rename("/newexistingname");
    EXPECT_EQ(2u, device().numBlocks()); // Only the blocks of one file are left
}

TEST_F(CryNodeTest, Rename_UpdatesParentPointers_File) {
    this->CreateDir("/mydir");
    auto node = this->CreateFile("/oldname");
    node->rename("/mydir/newname");
    EXPECT_TRUE(node->checkParentPointer());
}

TEST_F(CryNodeTest, Rename_UpdatesParentPointers_Dir) {
    this->CreateDir("/mydir");
    auto node = this->CreateDir("/oldname");
    node->rename("/mydir/newname");
    EXPECT_TRUE(node->checkParentPointer());
}

TEST_F(CryNodeTest, Rename_UpdatesParentPointers_Symlink) {
    this->CreateDir("/mydir");
    auto node = this->CreateSymlink("/oldname");
    node->rename("/mydir/newname");
    EXPECT_TRUE(node->checkParentPointer());
}
