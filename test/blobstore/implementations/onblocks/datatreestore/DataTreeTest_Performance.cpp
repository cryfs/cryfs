#include "testutils/DataTreeTest.h"

#include <gmock/gmock.h>

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;
using blockstore::testfake::FakeBlockStore;
using cpputils::Data;
using cpputils::make_unique_ref;

class DataTreeTest_Performance: public DataTreeTest {
public:

};

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByTree) {
    auto key = this->CreateFullTwoLevel()->key();
    auto tree = this->treeStore.load(key).value();
    this->treeStore.remove(std::move(tree));
    EXPECT_EQ(2u, blockStore->loadedBlocks.size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByKey) {
    auto key = this->CreateFullTwoLevel()->key();
    this->treeStore.remove(key);
    EXPECT_EQ(1u, blockStore->loadedBlocks.size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByTree) {
    auto key = this->CreateFullThreeLevel()->key();
    auto tree = this->treeStore.load(key).value();
    this->treeStore.remove(std::move(tree));
    EXPECT_EQ(2u + nodeStore->layout().maxChildrenPerInnerNode(), blockStore->loadedBlocks.size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByKey) {
    auto key = this->CreateFullThreeLevel()->key();
    this->treeStore.remove(key);
    EXPECT_EQ(1u + nodeStore->layout().maxChildrenPerInnerNode(), blockStore->loadedBlocks.size());
}