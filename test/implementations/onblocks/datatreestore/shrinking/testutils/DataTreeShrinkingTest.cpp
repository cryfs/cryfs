#include <messmer/blobstore/test/implementations/onblocks/datatreestore/shrinking/testutils/DataTreeShrinkingTest.h>

using namespace blobstore::onblocks::datanodestore;

using std::unique_ptr;
using std::make_unique;
using cpputils::dynamic_pointer_move;
using blockstore::Key;
using blobstore::onblocks::datatreestore::DataTree;

void DataTreeShrinkingTest::Shrink(const Key &key) {
  DataTree tree(&nodeStore, nodeStore.load(key));
  tree.removeLastDataLeaf();
}

Key DataTreeShrinkingTest::CreateTwoLeafTree() {
  auto leaf1 = nodeStore.createNewLeafNode();
  auto root = nodeStore.createNewInnerNode(*leaf1);
  root->addChild(*nodeStore.createNewLeafNode());
  return root->key();
}

Key DataTreeShrinkingTest::CreateFourNodeThreeLeafTree() {
  auto leaf1 = nodeStore.createNewLeafNode();
  auto root = nodeStore.createNewInnerNode(*leaf1);
  root->addChild(*nodeStore.createNewLeafNode());
  root->addChild(*nodeStore.createNewLeafNode());
  return root->key();
}

Key DataTreeShrinkingTest::CreateTwoInnerNodeOneTwoLeavesTree() {
  auto leaf1 = nodeStore.createNewLeafNode();
  auto node1 = nodeStore.createNewInnerNode(*leaf1);
  auto leaf2 = nodeStore.createNewLeafNode();
  auto node2 = nodeStore.createNewInnerNode(*leaf2);
  node2->addChild(*nodeStore.createNewLeafNode());
  auto root = nodeStore.createNewInnerNode(*node1);
  root->addChild(*node2);
  return root->key();
}

Key DataTreeShrinkingTest::CreateTwoInnerNodeTwoOneLeavesTree() {
  auto leaf1 = nodeStore.createNewLeafNode();
  auto node1 = nodeStore.createNewInnerNode(*leaf1);
  node1->addChild(*nodeStore.createNewLeafNode());
  auto leaf2 = nodeStore.createNewLeafNode();
  auto node2 = nodeStore.createNewInnerNode(*leaf2);
  auto root = nodeStore.createNewInnerNode(*node1);
  root->addChild(*node2);
  return root->key();
}

Key DataTreeShrinkingTest::CreateThreeLevelMinDataTree() {
  auto fullTwoLevelRoot = nodeStore.load(CreateFullTwoLevelTree());
  auto root = nodeStore.createNewInnerNode(*fullTwoLevelRoot);
  auto leaf = nodeStore.createNewLeafNode();
  auto inner = nodeStore.createNewInnerNode(*leaf);
  root->addChild(*inner);
  return root->key();
}

Key DataTreeShrinkingTest::CreateFourLevelMinDataTree() {
  auto fullThreeLevelRoot = nodeStore.load(CreateFullThreeLevelTree());
  auto root = nodeStore.createNewInnerNode(*fullThreeLevelRoot);
  auto leaf = nodeStore.createNewLeafNode();
  auto inner = nodeStore.createNewInnerNode(*leaf);
  auto nodechainRoot = nodeStore.createNewInnerNode(*inner);
  root->addChild(*nodechainRoot);
  return root->key();
}

Key DataTreeShrinkingTest::CreateFourLevelTreeWithTwoSiblingLeaves1() {
  auto fullThreeLevelRoot = nodeStore.load(CreateFullThreeLevelTree());
  auto root = nodeStore.createNewInnerNode(*fullThreeLevelRoot);
  auto leaf = nodeStore.createNewLeafNode();
  auto inner = nodeStore.createNewInnerNode(*leaf);
  inner->addChild(*nodeStore.createNewLeafNode());
  auto inner_top = nodeStore.createNewInnerNode(*inner);
  root->addChild(*inner_top);
  return root->key();
}

Key DataTreeShrinkingTest::CreateFourLevelTreeWithTwoSiblingLeaves2() {
  auto fullThreeLevelRoot = nodeStore.load(CreateFullThreeLevelTree());
  auto root = nodeStore.createNewInnerNode(*fullThreeLevelRoot);
  auto leaf1 = nodeStore.createNewLeafNode();
  auto inner1 = nodeStore.createNewInnerNode(*leaf1);
  FillNode(inner1.get());
  auto leaf2 = nodeStore.createNewLeafNode();
  auto inner2 = nodeStore.createNewInnerNode(*leaf2);
  inner2->addChild(*nodeStore.createNewLeafNode());
  auto inner_top = nodeStore.createNewInnerNode(*inner1);
  inner_top->addChild(*inner2);
  root->addChild(*inner_top);
  return root->key();
}

Key DataTreeShrinkingTest::CreateTreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel() {
  auto fullThreeLevelRoot = nodeStore.load(CreateFullThreeLevelTree());
  auto root = nodeStore.createNewInnerNode(*fullThreeLevelRoot);
  auto leaf1 = nodeStore.createNewLeafNode();
  auto inner1 = nodeStore.createNewInnerNode(*leaf1);
  FillNode(inner1.get());
  auto leaf2 = nodeStore.createNewLeafNode();
  auto inner2 = nodeStore.createNewInnerNode(*leaf2);
  auto inner_top = nodeStore.createNewInnerNode(*inner1);
  inner_top->addChild(*inner2);
  root->addChild(*inner_top);
  return root->key();
}

Key DataTreeShrinkingTest::CreateThreeLevelTreeWithThreeChildrenOfRoot() {
  auto fullTwoLevelTree1 = nodeStore.load(CreateFullTwoLevelTree());
  auto fullTwoLevelTree2 = nodeStore.load(CreateFullTwoLevelTree());
  auto twonodechain = nodeStore.createNewInnerNode(*nodeStore.createNewLeafNode());
  auto root = nodeStore.createNewInnerNode(*fullTwoLevelTree1);
  root->addChild(*fullTwoLevelTree2);
  root->addChild(*twonodechain);
  return root->key();
}
