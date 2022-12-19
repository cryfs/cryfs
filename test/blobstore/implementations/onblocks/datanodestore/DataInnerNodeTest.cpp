#include <gtest/gtest.h>

#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <blockstore/implementations/testfake/FakeBlock.h>

#include <memory>
#include <cpp-utils/pointer/cast.h>

using ::testing::Test;

using cpputils::dynamic_pointer_move;

using blockstore::BlockId;
using blockstore::testfake::FakeBlockStore;
using blockstore::BlockStore;
using cpputils::Data;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::vector;

class DataInnerNodeTest: public Test {
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;

  DataInnerNodeTest() :
    _blockStore(make_unique_ref<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique_ref<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES)),
    ZEROES(nodeStore->layout().maxBytesPerLeaf()),
    leaf(nodeStore->createNewLeafNode(Data(0))),
    node(nodeStore->createNewInnerNode(1, {leaf->blockId()})) {

    ZEROES.FillWithZeroes();
  }

  unique_ref<DataInnerNode> LoadInnerNode(const BlockId &blockId) {
    auto node = nodeStore->load(blockId).value();
    return dynamic_pointer_move<DataInnerNode>(node).value();
  }

  BlockId CreateNewInnerNodeReturnKey(const DataNode &firstChild) {
    return nodeStore->createNewInnerNode(firstChild.depth()+1, {firstChild.blockId()})->blockId();
  }

  unique_ref<DataInnerNode> CreateNewInnerNode() {
    auto new_leaf = nodeStore->createNewLeafNode(Data(0));
    return nodeStore->createNewInnerNode(1, {new_leaf->blockId()});
  }

  unique_ref<DataInnerNode> CreateAndLoadNewInnerNode(const DataNode &firstChild) {
    auto blockId = CreateNewInnerNodeReturnKey(firstChild);
    return LoadInnerNode(blockId);
  }

  unique_ref<DataInnerNode> CreateNewInnerNode(uint8_t depth, const vector<blockstore::BlockId> &children) {
    return nodeStore->createNewInnerNode(depth, children);
  }

  BlockId CreateNewInnerNodeReturnKey(uint8_t depth, const vector<blockstore::BlockId> &children) {
    return CreateNewInnerNode(depth, children)->blockId();
  }

  unique_ref<DataInnerNode> CreateAndLoadNewInnerNode(uint8_t depth, const vector<blockstore::BlockId> &children) {
    auto blockId = CreateNewInnerNodeReturnKey(depth, children);
    return LoadInnerNode(blockId);
  }

  BlockId AddALeafTo(DataInnerNode *node) {
    auto leaf2 = nodeStore->createNewLeafNode(Data(0));
    node->addChild(*leaf2);
    return leaf2->blockId();
  }

  BlockId CreateNodeWithDataConvertItToInnerNodeAndReturnKey() {
    auto node = CreateNewInnerNode();
    AddALeafTo(node.get());
    AddALeafTo(node.get());
    auto child = nodeStore->createNewLeafNode(Data(0));
    unique_ref<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(node), nodeStore->layout(), *child);
    return converted->blockId();
  }

  unique_ref<DataInnerNode> CopyInnerNode(const DataInnerNode &node) {
    auto copied = nodeStore->createNewNodeAsCopyFrom(node);
    return dynamic_pointer_move<DataInnerNode>(copied).value();
  }

  BlockId InitializeInnerNodeAddLeafReturnKey() {
    auto node = DataInnerNode::CreateNewNode(blockStore, nodeStore->layout(), 1, {leaf->blockId()});
    AddALeafTo(node.get());
    return node->blockId();
  }

  unique_ref<BlockStore> _blockStore;
  BlockStore *blockStore;
  unique_ref<DataNodeStore> nodeStore;
  Data ZEROES;
  unique_ref<DataLeafNode> leaf;
  unique_ref<DataInnerNode> node;

private:

  DISALLOW_COPY_AND_ASSIGN(DataInnerNodeTest);
};

constexpr uint32_t DataInnerNodeTest::BLOCKSIZE_BYTES;

TEST_F(DataInnerNodeTest, InitializesCorrectly) {
  auto node = DataInnerNode::CreateNewNode(blockStore, nodeStore->layout(), 1, {leaf->blockId()});

  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->blockId(), node->readChild(0).blockId());
}

TEST_F(DataInnerNodeTest, ReinitializesCorrectly) {
  auto blockId = DataLeafNode::CreateNewNode(blockStore, nodeStore->layout(), Data(0))->blockId();
  auto node = DataInnerNode::InitializeNewNode(blockStore->load(blockId).value(), nodeStore->layout(), 1, {leaf->blockId()});

  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->blockId(), node->readChild(0).blockId());
}

TEST_F(DataInnerNodeTest, IsCorrectlyInitializedAfterLoading) {
  auto loaded = CreateAndLoadNewInnerNode(*leaf);

  EXPECT_EQ(1u, loaded->numChildren());
  EXPECT_EQ(leaf->blockId(), loaded->readChild(0).blockId());
}

TEST_F(DataInnerNodeTest, AddingASecondLeaf) {
  BlockId leaf2_blockId = AddALeafTo(node.get());

  EXPECT_EQ(2u, node->numChildren());
  EXPECT_EQ(leaf->blockId(), node->readChild(0).blockId());
  EXPECT_EQ(leaf2_blockId, node->readChild(1).blockId());
}

TEST_F(DataInnerNodeTest, AddingASecondLeafAndReload) {
  auto leaf2 = nodeStore->createNewLeafNode(Data(0));
  auto loaded = CreateAndLoadNewInnerNode(1, {leaf->blockId(), leaf2->blockId()});

  EXPECT_EQ(2u, loaded->numChildren());
  EXPECT_EQ(leaf->blockId(), loaded->readChild(0).blockId());
  EXPECT_EQ(leaf2->blockId(), loaded->readChild(1).blockId());
}

TEST_F(DataInnerNodeTest, BuildingAThreeLevelTree) {
  auto node2 = CreateNewInnerNode();
  auto parent = CreateNewInnerNode(node->depth()+1, {node->blockId(), node2->blockId()});

  EXPECT_EQ(2u, parent->numChildren());
  EXPECT_EQ(node->blockId(), parent->readChild(0).blockId());
  EXPECT_EQ(node2->blockId(), parent->readChild(1).blockId());
}

TEST_F(DataInnerNodeTest, BuildingAThreeLevelTreeAndReload) {
  auto node2 = CreateNewInnerNode();
  auto parent = CreateAndLoadNewInnerNode(node->depth()+1, {node->blockId(), node2->blockId()});

  EXPECT_EQ(2u, parent->numChildren());
  EXPECT_EQ(node->blockId(), parent->readChild(0).blockId());
  EXPECT_EQ(node2->blockId(), parent->readChild(1).blockId());
}

TEST_F(DataInnerNodeTest, ConvertToInternalNode) {
  auto child = nodeStore->createNewLeafNode(Data(0));
  BlockId node_blockId = node->blockId();
  unique_ref<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(node), nodeStore->layout(), *child);

  EXPECT_EQ(1u, converted->numChildren());
  EXPECT_EQ(child->blockId(), converted->readChild(0).blockId());
  EXPECT_EQ(node_blockId, converted->blockId());
}

TEST_F(DataInnerNodeTest, ConvertToInternalNodeZeroesOutChildrenRegion) {
  BlockId blockId = CreateNodeWithDataConvertItToInnerNodeAndReturnKey();

  auto block = blockStore->load(blockId).value();
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), static_cast<const uint8_t*>(block->data())+DataNodeLayout::HEADERSIZE_BYTES+sizeof(DataInnerNode::ChildEntry), nodeStore->layout().maxBytesPerLeaf()-sizeof(DataInnerNode::ChildEntry)));
}

TEST_F(DataInnerNodeTest, CopyingCreatesNewNode) {
  auto copied = CopyInnerNode(*node);
  EXPECT_NE(node->blockId(), copied->blockId());
}

TEST_F(DataInnerNodeTest, CopyInnerNodeWithOneChild) {
  auto copied = CopyInnerNode(*node);

  EXPECT_EQ(node->numChildren(), copied->numChildren());
  EXPECT_EQ(node->readChild(0).blockId(), copied->readChild(0).blockId());
}

TEST_F(DataInnerNodeTest, CopyInnerNodeWithTwoChildren) {
  AddALeafTo(node.get());
  auto copied = CopyInnerNode(*node);

  EXPECT_EQ(node->numChildren(), copied->numChildren());
  EXPECT_EQ(node->readChild(0).blockId(), copied->readChild(0).blockId());
  EXPECT_EQ(node->readChild(1).blockId(), copied->readChild(1).blockId());
}

TEST_F(DataInnerNodeTest, LastChildWhenOneChild) {
  EXPECT_EQ(leaf->blockId(), node->readLastChild().blockId());
}

TEST_F(DataInnerNodeTest, LastChildWhenTwoChildren) {
  BlockId blockId = AddALeafTo(node.get());
  EXPECT_EQ(blockId, node->readLastChild().blockId());
}

TEST_F(DataInnerNodeTest, LastChildWhenThreeChildren) {
  AddALeafTo(node.get());
  BlockId blockId = AddALeafTo(node.get());
  EXPECT_EQ(blockId, node->readLastChild().blockId());
}
