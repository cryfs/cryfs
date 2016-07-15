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
    uint64_t maxBytesPerLeaf = nodeStore->layout().maxBytesPerLeaf();
};

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByTree) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    treeStore.remove(std::move(tree));

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByKey) {
    auto key = CreateFullTwoLevel()->key();
    blockStore->resetCounters();

    treeStore.remove(key);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByTree) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    treeStore.remove(std::move(tree));

    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode + maxChildrenPerInnerNode*maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByKey) {
    auto key = CreateFullThreeLevel()->key();
    blockStore->resetCounters();

    treeStore.remove(key);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode + maxChildrenPerInnerNode*maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_All) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 0, maxChildrenPerInnerNode);

    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Loads all leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_Some) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 3, 5);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Loads both leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_All) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 0, maxChildrenPerInnerNode * maxChildrenPerInnerNode);

    EXPECT_EQ(maxChildrenPerInnerNode + maxChildrenPerInnerNode * maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Loads inner nodes and all leaves once (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InOneInner) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 3, 5);

    EXPECT_EQ(3u, blockStore->loadedBlocks().size()); // Loads inner node and both leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InTwoInner) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 3, 3 + maxChildrenPerInnerNode);

    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Loads inner node and both leaves (not the root, because it is already loaded in the tree)
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_WholeInner) {
    auto key = CreateFullThreeLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, 2*maxChildrenPerInnerNode);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Loads inner node and leaves (not the root, because it is already loaded in the tree)f
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingInside) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 1, 4);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old child (for growing it)
    EXPECT_EQ(2u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add children to inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingOutside_TwoLevel) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 4, 5);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(3u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add child to inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingOutside_ThreeLevel) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 2*maxChildrenPerInnerNode+1, 2*maxChildrenPerInnerNode+2);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Loads last old leaf (and its inner node) for growing it
    EXPECT_EQ(3u, blockStore->createdBlocks()); // inner node and two leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingAtBeginOfChild) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, 3*maxChildrenPerInnerNode);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Inner node and its leaves
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // Creates an inner node and its leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInOldDepth) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 4, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // Add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInOldDepth_ResizeLastLeaf) {
    auto key = CreateInner({CreateLeaf(), CreateLeafWithSize(5)})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), 4, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // Resize last leaf and add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInNewDepth) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // Add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInNewDepth_ResizeLastLeaf) {
    auto key = CreateInner({CreateLeaf(), CreateLeafWithSize(5)})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    Traverse(tree.get(), maxChildrenPerInnerNode, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // Resize last leaf and add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ZeroToZero) {
    auto key = CreateLeafWithSize(0)->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(0);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowOneLeaf) {
    auto key = CreateLeafWithSize(0)->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(5);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ShrinkOneLeaf) {
    auto key = CreateLeafWithSize(5)->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(2);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ShrinkOneLeafToZero) {
    auto key = CreateLeafWithSize(5)->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(0);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowOneLeafInLargerTree) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf(), CreateLeafWithSize(5)})})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*(maxChildrenPerInnerNode+1)+6); // Grow by one byte

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Load inner node and leaf
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowByOneLeaf) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*2+1); // Grow by one byte

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(1u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add child to inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowByOneLeaf_GrowLastLeaf) {
    auto key = CreateInner({CreateLeaf(), CreateLeafWithSize(5)})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*2+1); // Grow by one byte

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(1u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // add child to inner node and resize old last leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ShrinkByOneLeaf) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(2*maxBytesPerLeaf-1);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // resize new last leaf and remove leaf from inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_IncreaseTreeDepth_0to1) {
    auto key = CreateLeaf()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf+1);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(2u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be an inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_IncreaseTreeDepth_1to2) {
    auto key = CreateFullTwoLevel()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*maxChildrenPerInnerNode+1);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // check whether we have to grow last leaf
    EXPECT_EQ(3u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be an inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_IncreaseTreeDepth_0to2) {
    auto key = CreateLeaf()->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*maxChildrenPerInnerNode+1);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(3u + maxChildrenPerInnerNode, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be an inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_DecreaseTreeDepth_1to0) {
    auto key = CreateInner({CreateLeaf(), CreateLeaf()})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // read content of first leaf and load first leaf to replace root with it
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(2u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be a leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_DecreaseTreeDepth_2to1) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*maxChildrenPerInnerNode);

    EXPECT_EQ(4u, blockStore->loadedBlocks().size()); // load new last leaf (+inner node), load second inner node to remove its subtree, then load first child of root to replace root with its child.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(3u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be a leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_DecreaseTreeDepth_2to0) {
    auto key = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})})->key();
    auto tree = treeStore.load(key).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf);

    EXPECT_EQ(5u, blockStore->loadedBlocks().size()); // load new last leaf (+inner node), load second inner node to remove its subtree, then 2x load first child of root to replace root with its child.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(3u + maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // 2x rewrite root node to be a leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}
