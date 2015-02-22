#include "DataTreeGrowingTest.h"
#include <messmer/cpp-utils/pointer.h>

using namespace blobstore::onblocks::datanodestore;

using std::unique_ptr;
using cpputils::dynamic_pointer_move;
using blockstore::Key;
using blobstore::onblocks::datatreestore::DataTree;

Key DataTreeGrowingTest::CreateTreeAddOneLeafReturnRootKey() {
  auto tree = CreateLeafOnlyTree();
  auto key = tree->key();
  tree->addDataLeaf();
  return key;
}

Key DataTreeGrowingTest::CreateTreeAddTwoLeavesReturnRootKey() {
  auto tree = CreateLeafOnlyTree();
  auto key = tree->key();
  tree->addDataLeaf();
  tree->addDataLeaf();
  return key;
}

Key DataTreeGrowingTest::CreateTreeAddThreeLeavesReturnRootKey() {
  auto tree = CreateLeafOnlyTree();
  auto key = tree->key();
  tree->addDataLeaf();
  tree->addDataLeaf();
  tree->addDataLeaf();
  return key;
}

Key DataTreeGrowingTest::CreateThreeNodeChainedTreeReturnRootKey() {
  auto leaf = nodeStore.createNewLeafNode();
  auto node = nodeStore.createNewInnerNode(*leaf);
  auto root = nodeStore.createNewInnerNode(*node);
  return root->key();
}

Key DataTreeGrowingTest::CreateThreeLevelTreeWithLowerLevelFullReturnRootKey() {
  auto leaf = nodeStore.createNewLeafNode();
  auto node = nodeStore.createNewInnerNode(*leaf);
  FillNode(node.get());
  auto root = nodeStore.createNewInnerNode(*node);
  return root->key();
}

Key DataTreeGrowingTest::CreateThreeLevelTreeWithTwoFullSubtrees() {
  auto leaf1 = nodeStore.createNewLeafNode();
  auto leaf2 = nodeStore.createNewLeafNode();
  auto leaf3 = nodeStore.createNewLeafNode();
  auto node1 = nodeStore.createNewInnerNode(*leaf1);
  FillNode(node1.get());
  auto node2 = nodeStore.createNewInnerNode(*leaf2);
  FillNode(node2.get());
  auto root = nodeStore.createNewInnerNode(*node1);
  root->addChild(*node2);
  return root->key();
}

void DataTreeGrowingTest::AddLeafTo(const Key &key) {
  DataTree tree(&nodeStore, nodeStore.load(key));
  tree.addDataLeaf();
}

void DataTreeGrowingTest::EXPECT_IS_THREENODE_CHAIN(const Key &key) {
  auto node1 = LoadInnerNode(key);
  EXPECT_EQ(1u, node1->numChildren());
  auto node2 = LoadInnerNode(node1->getChild(0)->key());
  EXPECT_EQ(1u, node2->numChildren());
  EXPECT_IS_LEAF_NODE(node2->getChild(0)->key());
}

void DataTreeGrowingTest::EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(unsigned int expectedNumberOfLeaves, const Key &key) {
  auto node = LoadInnerNode(key);
  EXPECT_EQ(expectedNumberOfLeaves, node->numChildren());
  for(unsigned int i=0;i<expectedNumberOfLeaves;++i) {
    EXPECT_IS_LEAF_NODE(node->getChild(i)->key());
  }
}
