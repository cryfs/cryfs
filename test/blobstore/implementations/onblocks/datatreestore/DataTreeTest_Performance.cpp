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
    void Traverse(DataTree *tree, uint64_t beginIndex, uint64_t endIndex) {
        tree->traverseLeaves(beginIndex, endIndex, [] (uint32_t /*index*/, DataLeafNode* /*leaf*/) {}, [this] (uint32_t /*index*/) -> Data {return Data(maxChildrenPerInnerNode).FillWithZeroes();});
    }

    uint64_t maxChildrenPerInnerNode = nodeStore->layout().maxChildrenPerInnerNode();
};

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByTree) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    treeStore.remove(std::move(tree));

    EXPECT_EQ(1u, blockStore->loadedBlocks.size()); // First loading is from loading the tree, second one from removing it (i.e. loading the root)
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByKey) {
    auto key = CreateFullTwoLevel()->key();
    blockStore->resetCounters();

    treeStore.remove(key);

    EXPECT_EQ(1u, blockStore->loadedBlocks.size());
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByTree) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    treeStore.remove(std::move(tree));

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks.size());
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByKey) {
    auto key = CreateFullThreeLevel()->key();
    blockStore->resetCounters();

    treeStore.remove(key);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks.size());
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_All) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 0, maxChildrenPerInnerNode);

    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->loadedBlocks.size()); // Loads all leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_Some) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 3, 5);

    EXPECT_EQ(2u, blockStore->loadedBlocks.size()); // Loads both leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_All) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 0, maxChildrenPerInnerNode * maxChildrenPerInnerNode);

    EXPECT_EQ(maxChildrenPerInnerNode + maxChildrenPerInnerNode * maxChildrenPerInnerNode, blockStore->loadedBlocks.size()); // Loads inner nodes and all leaves once (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InOneInner) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 3, 5);

    EXPECT_EQ(3u, blockStore->loadedBlocks.size()); // Loads inner node and both leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InTwoInner) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 3, 3 + maxChildrenPerInnerNode);

    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->loadedBlocks.size()); // Loads inner node and both leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_WholeInner) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, 2*maxChildrenPerInnerNode);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks.size()); // Loads inner node and leaves (not the root, because it is already loaded in the tree)f
    EXPECT_EQ(0u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingInside) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 1, 4);

    EXPECT_EQ(1u, blockStore->loadedBlocks.size()); // Loads last old child (for growing it)
    EXPECT_EQ(2u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingOutside_TwoLevel) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 4, 5);

    EXPECT_EQ(1u, blockStore->loadedBlocks.size()); // Loads last old leaf for growing it
    EXPECT_EQ(3u, blockStore->createdBlocks);
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingOutside_ThreeLevel) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 2*maxChildrenPerInnerNode+1, 2*maxChildrenPerInnerNode+2);

    EXPECT_EQ(2u, blockStore->loadedBlocks.size()); // Loads last old leaf (and its inner node) for growing it
    EXPECT_EQ(3u, blockStore->createdBlocks); // inner node and two leaves
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingAtBeginOfChild) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, 3*maxChildrenPerInnerNode);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks.size()); // Inner node and its leaves
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->createdBlocks); // Creates an inner node and its leaves
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInOldDepth) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 4, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks.size()); // Loads last old leaf for growing it
    EXPECT_EQ(2 + maxChildrenPerInnerNode, blockStore->createdBlocks); // 2x new inner node + leaves
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInNewDepth) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks.size()); // Loads last old leaf for growing it
    EXPECT_EQ(2 + maxChildrenPerInnerNode, blockStore->createdBlocks); // 2x new inner node + leaves
}
