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
  return CreateInner({CreateInner({CreateLeaf()})})->key();
}

Key DataTreeGrowingTest::CreateThreeLevelTreeWithLowerLevelFullReturnRootKey() {
  return CreateInner({CreateFullTwoLevel()})->key();
}

Key DataTreeGrowingTest::CreateThreeLevelTreeWithTwoFullSubtrees() {
  return CreateInner({CreateFullTwoLevel(), CreateFullTwoLevel()})->key();
}

void DataTreeGrowingTest::AddLeafTo(const Key &key) {
  treeStore.load(key)->addDataLeaf();
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
