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

using blockstore::Key;
using blockstore::testfake::FakeBlockStore;
using blockstore::BlockStore;
using cpputils::Data;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

using cpputils::unique_ref;
using cpputils::make_unique_ref;

class DataInnerNodeTest: public Test {
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;

  DataInnerNodeTest() :
    _blockStore(make_unique_ref<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique_ref<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES)),
    ZEROES(nodeStore->layout().maxBytesPerLeaf()),
    leaf(nodeStore->createNewLeafNode()),
    node(nodeStore->createNewInnerNode(*leaf)) {

    ZEROES.FillWithZeroes();
  }

  unique_ref<DataInnerNode> LoadInnerNode(const Key &key) {
    auto node = nodeStore->load(key).value();
    return dynamic_pointer_move<DataInnerNode>(node).value();
  }

  Key CreateNewInnerNodeReturnKey(const DataNode &firstChild) {
    return nodeStore->createNewInnerNode(firstChild)->key();
  }

  unique_ref<DataInnerNode> CreateNewInnerNode() {
    auto new_leaf = nodeStore->createNewLeafNode();
    return nodeStore->createNewInnerNode(*new_leaf);
  }

  unique_ref<DataInnerNode> CreateAndLoadNewInnerNode(const DataNode &firstChild) {
    auto key = CreateNewInnerNodeReturnKey(firstChild);
    return LoadInnerNode(key);
  }

  unique_ref<DataInnerNode> CreateNewInnerNode(const DataNode &firstChild, const DataNode &secondChild) {
    auto node = nodeStore->createNewInnerNode(firstChild);
    node->addChild(secondChild);
    return node;
  }

  Key CreateNewInnerNodeReturnKey(const DataNode &firstChild, const DataNode &secondChild) {
    return CreateNewInnerNode(firstChild, secondChild)->key();
  }

  unique_ref<DataInnerNode> CreateAndLoadNewInnerNode(const DataNode &firstChild, const DataNode &secondChild) {
    auto key = CreateNewInnerNodeReturnKey(firstChild, secondChild);
    return LoadInnerNode(key);
  }

  Key AddALeafTo(DataInnerNode *node) {
    auto leaf2 = nodeStore->createNewLeafNode();
    node->addChild(*leaf2);
    return leaf2->key();
  }

  Key CreateNodeWithDataConvertItToInnerNodeAndReturnKey() {
    auto node = CreateNewInnerNode();
    AddALeafTo(node.get());
    AddALeafTo(node.get());
    auto child = nodeStore->createNewLeafNode();
    unique_ref<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(node), *child);
    return converted->key();
  }

  unique_ref<DataInnerNode> CopyInnerNode(const DataInnerNode &node) {
    auto copied = nodeStore->createNewNodeAsCopyFrom(node);
    return dynamic_pointer_move<DataInnerNode>(copied).value();
  }

  Key InitializeInnerNodeAddLeafReturnKey() {
    auto node = DataInnerNode::InitializeNewNode(blockStore->create(Data(BLOCKSIZE_BYTES)), *leaf);
    AddALeafTo(node.get());
    return node->key();
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

TEST_F(DataInnerNodeTest, CorrectKeyReturnedAfterInitialization) {
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  Key key = block->key();
  auto node = DataInnerNode::InitializeNewNode(std::move(block), *leaf);
  EXPECT_EQ(key, node->key());
}

TEST_F(DataInnerNodeTest, CorrectKeyReturnedAfterLoading) {
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  Key key = block->key();
  DataInnerNode::InitializeNewNode(std::move(block), *leaf);

  auto loaded = nodeStore->load(key).value();
  EXPECT_EQ(key, loaded->key());
}

TEST_F(DataInnerNodeTest, InitializesCorrectly) {
  auto node = DataInnerNode::InitializeNewNode(blockStore->create(Data(BLOCKSIZE_BYTES)), *leaf);

  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, ReinitializesCorrectly) {
  auto key = InitializeInnerNodeAddLeafReturnKey();
  auto node = DataInnerNode::InitializeNewNode(blockStore->load(key).value(), *leaf);

  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, IsCorrectlyInitializedAfterLoading) {
  auto loaded = CreateAndLoadNewInnerNode(*leaf);

  EXPECT_EQ(1u, loaded->numChildren());
  EXPECT_EQ(leaf->key(), loaded->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, AddingASecondLeaf) {
  Key leaf2_key = AddALeafTo(node.get());

  EXPECT_EQ(2u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
  EXPECT_EQ(leaf2_key, node->getChild(1)->key());
}

TEST_F(DataInnerNodeTest, AddingASecondLeafAndReload) {
  auto leaf2 = nodeStore->createNewLeafNode();
  auto loaded = CreateAndLoadNewInnerNode(*leaf, *leaf2);

  EXPECT_EQ(2u, loaded->numChildren());
  EXPECT_EQ(leaf->key(), loaded->getChild(0)->key());
  EXPECT_EQ(leaf2->key(), loaded->getChild(1)->key());
}

TEST_F(DataInnerNodeTest, BuildingAThreeLevelTree) {
  auto node2 = CreateNewInnerNode();
  auto parent = CreateNewInnerNode(*node, *node2);

  EXPECT_EQ(2u, parent->numChildren());
  EXPECT_EQ(node->key(), parent->getChild(0)->key());
  EXPECT_EQ(node2->key(), parent->getChild(1)->key());
}

TEST_F(DataInnerNodeTest, BuildingAThreeLevelTreeAndReload) {
  auto node2 = CreateNewInnerNode();
  auto parent = CreateAndLoadNewInnerNode(*node, *node2);

  EXPECT_EQ(2u, parent->numChildren());
  EXPECT_EQ(node->key(), parent->getChild(0)->key());
  EXPECT_EQ(node2->key(), parent->getChild(1)->key());
}

TEST_F(DataInnerNodeTest, ConvertToInternalNode) {
  auto child = nodeStore->createNewLeafNode();
  Key node_key = node->key();
  unique_ref<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(node), *child);

  EXPECT_EQ(1u, converted->numChildren());
  EXPECT_EQ(child->key(), converted->getChild(0)->key());
  EXPECT_EQ(node_key, converted->key());
}

TEST_F(DataInnerNodeTest, ConvertToInternalNodeZeroesOutChildrenRegion) {
  Key key = CreateNodeWithDataConvertItToInnerNodeAndReturnKey();

  auto block = blockStore->load(key).value();
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), (uint8_t*)block->data()+DataNodeLayout::HEADERSIZE_BYTES+sizeof(DataInnerNode::ChildEntry), nodeStore->layout().maxBytesPerLeaf()-sizeof(DataInnerNode::ChildEntry)));
}

TEST_F(DataInnerNodeTest, CopyingCreatesNewNode) {
  auto copied = CopyInnerNode(*node);
  EXPECT_NE(node->key(), copied->key());
}

TEST_F(DataInnerNodeTest, CopyInnerNodeWithOneChild) {
  auto copied = CopyInnerNode(*node);

  EXPECT_EQ(node->numChildren(), copied->numChildren());
  EXPECT_EQ(node->getChild(0)->key(), copied->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, CopyInnerNodeWithTwoChildren) {
  AddALeafTo(node.get());
  auto copied = CopyInnerNode(*node);

  EXPECT_EQ(node->numChildren(), copied->numChildren());
  EXPECT_EQ(node->getChild(0)->key(), copied->getChild(0)->key());
  EXPECT_EQ(node->getChild(1)->key(), copied->getChild(1)->key());
}

TEST_F(DataInnerNodeTest, LastChildWhenOneChild) {
  EXPECT_EQ(leaf->key(), node->LastChild()->key());
}

TEST_F(DataInnerNodeTest, LastChildWhenTwoChildren) {
  Key key = AddALeafTo(node.get());
  EXPECT_EQ(key, node->LastChild()->key());
}

TEST_F(DataInnerNodeTest, LastChildWhenThreeChildren) {
  AddALeafTo(node.get());
  Key key = AddALeafTo(node.get());
  EXPECT_EQ(key, node->LastChild()->key());
}
