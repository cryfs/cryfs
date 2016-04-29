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
        auto _parentDir = device().Load(path.parent_path()).value();
        auto parentDir = dynamic_pointer_move<CryDir>(_parentDir).value();
        parentDir->createAndOpenFile(path.filename().native(), MODE_PUBLIC, 0, 0);
        auto createdFile = device().Load(path).value();
        return dynamic_pointer_move<CryNode>(createdFile).value();
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
    auto node = CreateFile("/oldname");
    CreateFile("/newexistingname");
    EXPECT_EQ(3u, device().numBlocks()); // In the beginning, there is three blocks (the root block and the two created files). If that is not true anymore, we'll have to adapt the test case.
    node->rename("/newexistingname");
    EXPECT_EQ(2u, device().numBlocks()); // Only the blocks of one file are left
}
