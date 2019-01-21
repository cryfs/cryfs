#include "testutils/DataTreeTest.h"

#include <gmock/gmock.h>

using blobstore::onblocks::datatreestore::DataTree;
using blockstore::BlockId;
using cpputils::Data;

class DataTreeTest_Performance: public DataTreeTest {
public:
    void TraverseByWriting(DataTree *tree, uint64_t beginIndex, uint64_t endIndex) {
        uint64_t offset = beginIndex * maxBytesPerLeaf;
        uint64_t count = endIndex * maxBytesPerLeaf - offset;
        Data data(count);
        data.FillWithZeroes();
        tree->writeBytes(data.data(), offset, count);
    }

    void TraverseByReading(DataTree *tree, uint64_t beginIndex, uint64_t endIndex) {
        uint64_t offset = beginIndex * maxBytesPerLeaf;
        uint64_t count = endIndex * maxBytesPerLeaf - offset;
        Data data(count);
        tree->readBytes(data.data(), offset, count);
    }

    uint64_t maxChildrenPerInnerNode = nodeStore->layout().maxChildrenPerInnerNode();
    uint64_t maxBytesPerLeaf = nodeStore->layout().maxBytesPerLeaf();
};

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByTree) {
    auto blockId = CreateFullTwoLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    treeStore.remove(std::move(tree));

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Twolevel_DeleteByKey) {
    auto blockId = CreateFullTwoLevel()->blockId();
    blockStore->resetCounters();

    treeStore.remove(blockId);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByTree) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    treeStore.remove(std::move(tree));

    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode + maxChildrenPerInnerNode*maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, DeletingDoesntLoadLeaves_Threelevel_DeleteByKey) {
    auto blockId = CreateFullThreeLevel()->blockId();
    blockStore->resetCounters();

    treeStore.remove(blockId);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u + maxChildrenPerInnerNode + maxChildrenPerInnerNode*maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_All_ByWriting) {
    auto blockId = CreateFullTwoLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 0, maxChildrenPerInnerNode);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Has to load the rightmost leaf once to adapt its size, rest of the leaves aren't loaded but just overwritten
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_All_ByReading) {
    auto blockId = CreateFullTwoLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByReading(tree.get(), 0, maxChildrenPerInnerNode);

    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size());  // Has to read the rightmost leaf an additional time in the beginning to determine size.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_Some_ByWriting) {
    auto blockId = CreateFullTwoLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 3, 5);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Twolevel_Some_ByReading) {
    auto blockId = CreateFullTwoLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByReading(tree.get(), 3, 5);

    EXPECT_EQ(3u, blockStore->loadedBlocks().size());  // reads 2 leaves and the rightmost leaf to determine size
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_All_ByWriting) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 0, maxChildrenPerInnerNode * maxChildrenPerInnerNode);

    EXPECT_EQ(maxChildrenPerInnerNode + 1, blockStore->loadedBlocks().size()); // Loads inner nodes and has to load the rightmost leaf once to adapt its size, rest of the leaves aren't loaded but just overwritten.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(maxChildrenPerInnerNode*maxChildrenPerInnerNode, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_All_ByReading) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByReading(tree.get(), 0, maxChildrenPerInnerNode * maxChildrenPerInnerNode);

    EXPECT_EQ(maxChildrenPerInnerNode*maxChildrenPerInnerNode + maxChildrenPerInnerNode + 2, blockStore->loadedBlocks().size()); // Loads inner nodes and leaves. Has to load the rightmost inner node and leaf an additional time at the beginning to compute size
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InOneInner_ByWriting) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 3, 5);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads inner node. Doesn't load the leaves, they're just overwritten.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InOneInner_ByReading) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByReading(tree.get(), 3, 5);

    EXPECT_EQ(5u, blockStore->loadedBlocks().size());  // reads 2 leaves and the inner node, also has to read the rightmost inner node and leaf additionally at the beginning to determine size
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InTwoInner_ByWriting) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 3, 3 + maxChildrenPerInnerNode);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Loads both inner node
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_InTwoInner_ByReading) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByReading(tree.get(), 3, 3 + maxChildrenPerInnerNode);

    EXPECT_EQ(4u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Loads both inner nodes and the requested leaves. Also has to load rightmost inner node and leaf additionally in the beginning to determine size.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_WholeInner_ByWriting) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), maxChildrenPerInnerNode, 2*maxChildrenPerInnerNode);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads inner node. Doesn't load the leaves, they're just overwritten.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(maxChildrenPerInnerNode, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_Threelevel_WholeInner_ByReading) {
    auto blockId = CreateFullThreeLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByReading(tree.get(), maxChildrenPerInnerNode, 2*maxChildrenPerInnerNode);

    EXPECT_EQ(3u + maxChildrenPerInnerNode, blockStore->loadedBlocks().size()); // Loads inner node and all requested leaves. Also has to load rightmost inner node and leaf additionally in the beginning to determine size.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingInside) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 1, 4);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old child (for growing it)
    EXPECT_EQ(2u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // write the data and add children to inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingOutside_TwoLevel) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 4, 5);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(3u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add child to inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingOutside_ThreeLevel) {
    auto blockId = CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 2*maxChildrenPerInnerNode+1, 2*maxChildrenPerInnerNode+2);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Loads last old leaf (and its inner node) for growing it
    EXPECT_EQ(3u, blockStore->createdBlocks()); // inner node and two leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTree_StartingAtBeginOfChild) {
    auto blockId = CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), maxChildrenPerInnerNode, 3*maxChildrenPerInnerNode);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Loads inner node and one leaf to check whether we have to grow it. Doesn't load the leaves, but returns the keys of the leaves to the callback.
    EXPECT_EQ(1u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // Creates an inner node and its leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(maxChildrenPerInnerNode + 1u, blockStore->distinctWrittenBlocks().size()); // write data and add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInOldDepth) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 4, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // Add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInOldDepth_ResizeLastLeaf) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeafWithSize(5)})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), 4, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // Resize last leaf and add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInNewDepth) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), maxChildrenPerInnerNode, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // Add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, TraverseLeaves_GrowingTreeDepth_StartingInNewDepth_ResizeLastLeaf) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeafWithSize(5)})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    TraverseByWriting(tree.get(), maxChildrenPerInnerNode, maxChildrenPerInnerNode+2);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // Loads last old leaf for growing it
    EXPECT_EQ(2u + maxChildrenPerInnerNode, blockStore->createdBlocks()); // 2x new inner node + leaves
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // Resize last leaf and add children to existing inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ZeroToZero) {
    auto blockId = CreateLeafWithSize(0)->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(0);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(0u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowOneLeaf) {
    auto blockId = CreateLeafWithSize(0)->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(5);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ShrinkOneLeaf) {
    auto blockId = CreateLeafWithSize(5)->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(2);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ShrinkOneLeafToZero) {
    auto blockId = CreateLeafWithSize(5)->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(0);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowOneLeafInLargerTree) {
    auto blockId = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf(), CreateLeafWithSize(5)})})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*(maxChildrenPerInnerNode+1)+6); // Grow by one byte

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // Load inner node and leaf
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size());
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowByOneLeaf) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*2+1); // Grow by one byte

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(1u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // add child to inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_GrowByOneLeaf_GrowLastLeaf) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeafWithSize(5)})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*2+1); // Grow by one byte

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(1u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // add child to inner node and resize old last leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_ShrinkByOneLeaf) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(2*maxBytesPerLeaf-1);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size());
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(1u, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // resize new last leaf and remove leaf from inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_IncreaseTreeDepth_0to1) {
    auto blockId = CreateLeaf()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf+1);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(2u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be an inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_IncreaseTreeDepth_1to2) {
    auto blockId = CreateFullTwoLevel()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*maxChildrenPerInnerNode+1);

    EXPECT_EQ(1u, blockStore->loadedBlocks().size()); // check whether we have to grow last leaf
    EXPECT_EQ(3u, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be an inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_IncreaseTreeDepth_0to2) {
    auto blockId = CreateLeaf()->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*maxChildrenPerInnerNode+1);

    EXPECT_EQ(0u, blockStore->loadedBlocks().size());
    EXPECT_EQ(3u + maxChildrenPerInnerNode, blockStore->createdBlocks());
    EXPECT_EQ(0u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be an inner node
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_DecreaseTreeDepth_1to0) {
    auto blockId = CreateInner({CreateLeaf(), CreateLeaf()})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf);

    EXPECT_EQ(2u, blockStore->loadedBlocks().size()); // read content of first leaf and load first leaf to replace root with it
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(2u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be a leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_DecreaseTreeDepth_2to1) {
    auto blockId = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf*maxChildrenPerInnerNode);

    EXPECT_EQ(4u, blockStore->loadedBlocks().size()); // load new last leaf (+inner node), load second inner node to remove its subtree, then load first child of root to replace root with its child.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(3u, blockStore->removedBlocks().size());
    EXPECT_EQ(1u, blockStore->distinctWrittenBlocks().size()); // rewrite root node to be a leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}

TEST_F(DataTreeTest_Performance, ResizeNumBytes_DecreaseTreeDepth_2to0) {
    auto blockId = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})})->blockId();
    auto tree = treeStore.load(blockId).value();
    blockStore->resetCounters();

    tree->resizeNumBytes(maxBytesPerLeaf);

    EXPECT_EQ(5u, blockStore->loadedBlocks().size()); // load new last leaf (+inner node), load second inner node to remove its subtree, then 2x load first child of root to replace root with its child.
    EXPECT_EQ(0u, blockStore->createdBlocks());
    EXPECT_EQ(3u + maxChildrenPerInnerNode, blockStore->removedBlocks().size());
    EXPECT_EQ(2u, blockStore->distinctWrittenBlocks().size()); // remove children from inner node and rewrite root node to be a leaf
    EXPECT_EQ(0u, blockStore->resizedBlocks().size());
}
