#include "DataTreeTest.h"

#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"
#include <messmer/cpp-utils/pointer.h>

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using std::make_unique;
using std::unique_ptr;
using cpputils::dynamic_pointer_move;

DataTreeTest::DataTreeTest()
  :nodeStore(make_unique<FakeBlockStore>()) {
}

unique_ptr<DataTree> DataTreeTest::CreateLeafOnlyTree() {
  auto leafnode = nodeStore.createNewLeafNode();
  return make_unique<DataTree>(&nodeStore, std::move(leafnode));
}

void DataTreeTest::FillNode(DataInnerNode *node) {
  for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    node->addChild(*nodeStore.createNewLeafNode());
  }
}

void DataTreeTest::FillNodeTwoLevel(DataInnerNode *node) {
  for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    auto inner_node = nodeStore.createNewInnerNode(*nodeStore.createNewLeafNode());
    for(unsigned int j = 1;j < DataInnerNode::MAX_STORED_CHILDREN; ++j) {
      inner_node->addChild(*nodeStore.createNewLeafNode());
    }
    node->addChild(*inner_node);
  }
}

Key DataTreeTest::CreateFullTwoLevelTree() {
  auto leaf = nodeStore.createNewLeafNode();
  auto root = nodeStore.createNewInnerNode(*leaf);
  FillNode(root.get());
  return root->key();
}

Key DataTreeTest::CreateFullThreeLevelTree() {
  auto leaf = nodeStore.createNewLeafNode();
  auto node = nodeStore.createNewInnerNode(*leaf);
  auto root = nodeStore.createNewInnerNode(*node);
  FillNode(node.get());
  FillNodeTwoLevel(root.get());
  return root->key();
}

unique_ptr<DataInnerNode> DataTreeTest::LoadInnerNode(const Key &key) {
  auto node = nodeStore.load(key);
  auto casted = dynamic_pointer_move<DataInnerNode>(node);
  EXPECT_NE(nullptr, casted.get()) << "Is not an inner node";
  return casted;
}

unique_ptr<DataLeafNode> DataTreeTest::LoadLeafNode(const Key &key) {
  auto node = nodeStore.load(key);
  auto casted =  dynamic_pointer_move<DataLeafNode>(node);
  EXPECT_NE(nullptr, casted.get()) << "Is not a leaf node";
  return casted;
}

void DataTreeTest::EXPECT_IS_LEAF_NODE(const Key &key) {
  auto node = LoadLeafNode(key);
  EXPECT_NE(nullptr, node.get());
}

void DataTreeTest::EXPECT_IS_INNER_NODE(const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_NE(nullptr, node.get());
}

void DataTreeTest::EXPECT_IS_TWONODE_CHAIN(const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_EQ(1u, node->numChildren());
  EXPECT_IS_LEAF_NODE(node->getChild(0)->key());
}

void DataTreeTest::EXPECT_IS_FULL_TWOLEVEL_TREE(const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_EQ(DataInnerNode::MAX_STORED_CHILDREN, node->numChildren());
  for (unsigned int i = 0; i < node->numChildren(); ++i) {
    EXPECT_IS_LEAF_NODE(node->getChild(i)->key());
  }
}

void DataTreeTest::EXPECT_IS_FULL_THREELEVEL_TREE(const Key &key) {
  auto root = LoadInnerNode(key);
  EXPECT_EQ(DataInnerNode::MAX_STORED_CHILDREN, root->numChildren());
  for (unsigned int i = 0; i < root->numChildren(); ++i) {
    auto node = LoadInnerNode(root->getChild(i)->key());
    EXPECT_EQ(DataInnerNode::MAX_STORED_CHILDREN, node->numChildren());
    for (unsigned int j = 0; j < node->numChildren(); ++j) {
      EXPECT_IS_LEAF_NODE(node->getChild(j)->key());
    }
  }
}
