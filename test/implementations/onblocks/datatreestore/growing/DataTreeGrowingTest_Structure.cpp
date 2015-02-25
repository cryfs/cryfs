#include "testutils/DataTreeGrowingTest.h"

using blobstore::onblocks::datanodestore::DataInnerNode;
using blockstore::Key;

class DataTreeGrowingTest_Structure: public DataTreeGrowingTest {};

TEST_F(DataTreeGrowingTest_Structure, GrowAOneNodeTree) {
  auto key = CreateTreeAddOneLeafReturnRootKey();
  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(2, key);
}

TEST_F(DataTreeGrowingTest_Structure, GrowATwoNodeTree) {
  auto key = CreateTreeAddTwoLeavesReturnRootKey();
  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(3, key);
}

TEST_F(DataTreeGrowingTest_Structure, GrowATwoLevelThreeNodeTree) {
  auto key = CreateTreeAddThreeLeavesReturnRootKey();
  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(4, key);
}

TEST_F(DataTreeGrowingTest_Structure, GrowAThreeNodeChainedTree) {
  auto key = CreateThreeNodeChainedTreeReturnRootKey();
  AddLeafTo(key);

  auto root = LoadInnerNode(key);
  EXPECT_EQ(1u, root->numChildren());

  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(2, root->getChild(0)->key());
}

TEST_F(DataTreeGrowingTest_Structure, GrowAFullTwoLevelTreeFromGroundUp) {
  auto key = CreateLeafOnlyTree()->key();
  for (int i = 1; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    AddLeafTo(key);
  }
  EXPECT_IS_FULL_TWOLEVEL_TREE(key);
}

TEST_F(DataTreeGrowingTest_Structure, GrowAFullTwoLevelTree) {
  auto root_key = CreateFullTwoLevel()->key();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_TWONODE_CHAIN(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest_Structure, GrowAThreeLevelTreeWithLowerLevelFull) {
  auto root_key = CreateThreeLevelTreeWithLowerLevelFullReturnRootKey();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_TWONODE_CHAIN(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest_Structure, GrowAFullThreeLevelTreeFromGroundUp) {
  auto key = CreateLeafOnlyTree()->key();
  for (int i = 1; i < nodeStore->layout().maxChildrenPerInnerNode() * nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    AddLeafTo(key);
  }
  EXPECT_IS_FULL_THREELEVEL_TREE(key);
}

TEST_F(DataTreeGrowingTest_Structure, GrowAFullThreeLevelTree) {
  auto root_key = CreateFullThreeLevel()->key();
  AddLeafTo(root_key);

  auto root = LoadInnerNode(root_key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_THREELEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_THREENODE_CHAIN(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest_Structure, GrowAThreeLevelTreeWithTwoFullSubtreesFromGroundUp) {
  auto key = CreateLeafOnlyTree()->key();
  for (int i = 1; i < 2 * nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    AddLeafTo(key);
  }

  auto root = LoadInnerNode(key);
  EXPECT_EQ(2u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(1)->key());
}

TEST_F(DataTreeGrowingTest_Structure, GrowAThreeLevelTreeWithTwoFullSubtrees) {
  auto key = CreateThreeLevelTreeWithTwoFullSubtrees();
  AddLeafTo(key);

  auto root = LoadInnerNode(key);
  EXPECT_EQ(3u, root->numChildren());

  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
  EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(1)->key());
  EXPECT_IS_TWONODE_CHAIN(root->getChild(2)->key());
}
