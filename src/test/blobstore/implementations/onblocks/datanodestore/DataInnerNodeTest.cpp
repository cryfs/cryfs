#include <gtest/gtest.h>

#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlock.h"

#include <memory>
#include "fspp/utils/pointer.h"

using ::testing::Test;

using fspp::dynamic_pointer_move;

using blockstore::Key;
using blockstore::testfake::FakeBlockStore;
using blockstore::BlockStore;
using blockstore::Data;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

using std::unique_ptr;
using std::make_unique;

class DataInnerNodeTest: public Test {
public:
  DataInnerNodeTest() :
    ZEROES(DataLeafNode::MAX_STORED_BYTES),
    _blockStore(make_unique<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique<DataNodeStore>(std::move(_blockStore))),
    leaf(nodeStore->createNewLeafNode()),
    node(nodeStore->createNewInnerNode(*leaf)) {

    ZEROES.FillWithZeroes();
  }

  unique_ptr<DataInnerNode> LoadInnerNode(const Key &key) {
    auto node = nodeStore->load(key);
    return dynamic_pointer_move<DataInnerNode>(node);
  }

  Key CreateNewInnerNodeReturnKey(const DataNode &firstChild) {
    return nodeStore->createNewInnerNode(firstChild)->key();
  }

  unique_ptr<DataInnerNode> CreateNewInnerNode() {
    auto new_leaf = nodeStore->createNewLeafNode();
    return nodeStore->createNewInnerNode(*new_leaf);
  }

  unique_ptr<DataInnerNode> CreateAndLoadNewInnerNode(const DataNode &firstChild) {
    auto key = CreateNewInnerNodeReturnKey(firstChild);
    return LoadInnerNode(key);
  }

  unique_ptr<DataInnerNode> CreateNewInnerNode(const DataNode &firstChild, const DataNode &secondChild) {
    auto node = nodeStore->createNewInnerNode(firstChild);
    node->addChild(secondChild);
    return node;
  }

  Key CreateNewInnerNodeReturnKey(const DataNode &firstChild, const DataNode &secondChild) {
    return CreateNewInnerNode(firstChild, secondChild)->key();
  }

  unique_ptr<DataInnerNode> CreateAndLoadNewInnerNode(const DataNode &firstChild, const DataNode &secondChild) {
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
    unique_ptr<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(node), *child);
    return converted->key();
  }

  Data ZEROES;
  unique_ptr<BlockStore> _blockStore;
  BlockStore *blockStore;
  unique_ptr<DataNodeStore> nodeStore;
  unique_ptr<DataLeafNode> leaf;
  unique_ptr<DataInnerNode> node;
};

TEST_F(DataInnerNodeTest, InitializesCorrectly) {
  node->InitializeNewNode(*leaf);

  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, ReinitializesCorrectly) {
  AddALeafTo(node.get());
  node->InitializeNewNode(*leaf);

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
  unique_ptr<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(node), *child);

  EXPECT_EQ(1u, converted->numChildren());
  EXPECT_EQ(child->key(), converted->getChild(0)->key());
  EXPECT_EQ(node_key, converted->key());
}

TEST_F(DataInnerNodeTest, ConvertToInternalNodeZeroesOutChildrenRegion) {
  Key key = CreateNodeWithDataConvertItToInnerNodeAndReturnKey();

  auto block = blockStore->load(key);
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), (uint8_t*)block->data()+DataNodeView::HEADERSIZE_BYTES+sizeof(DataInnerNode::ChildEntry), DataLeafNode::MAX_STORED_BYTES-sizeof(DataInnerNode::ChildEntry)));
}

//TODO TestCase for LastChild

