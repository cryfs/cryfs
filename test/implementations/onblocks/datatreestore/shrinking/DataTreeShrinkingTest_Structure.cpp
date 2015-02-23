#include "testutils/DataTreeShrinkingTest.h"

using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

class DataTreeShrinkingTest_Structure: public DataTreeShrinkingTest {
public:
  void EXPECT_IS_LEAF_ONLY_TREE(const Key &key) {
    EXPECT_IS_LEAF_NODE(key);
  }

  void EXPECT_IS_TWO_LEAF_TREE(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_LEAF_NODE(root->getChild(0)->key());
    EXPECT_IS_LEAF_NODE(root->getChild(1)->key());
  }

  void EXPECT_IS_TWO_INNER_NODE_TREE_WITH_ONE_LEAF_EACH(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_TWONODE_CHAIN(root->getChild(0)->key());
    EXPECT_IS_TWONODE_CHAIN(root->getChild(1)->key());
  }

  void EXPECT_IS_THREE_NODE_CHAIN(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(1, root->numChildren());
    EXPECT_IS_TWONODE_CHAIN(root->getChild(0)->key());
  }

  void EXPECT_IS_THREELEVEL_MINDATA_TREE(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
    EXPECT_IS_TWONODE_CHAIN(root->getChild(1)->key());
  }

  void EXPECT_IS_FOURLEVEL_MINDATA_TREE(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_FULL_THREELEVEL_TREE(root->getChild(0)->key());
    EXPECT_IS_THREE_NODE_CHAIN(root->getChild(1)->key());
  }

  void EXPECT_IS_TREE_WITH_FIRST_CHILD_OF_ROOT_FULL_THREELEVEL_AND_SECOND_CHILD_MINDATA_THREELEVEL_TREE(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_FULL_THREELEVEL_TREE(root->getChild(0)->key());
    EXPECT_IS_THREELEVEL_MINDATA_TREE(root->getChild(1)->key());
  }

  void EXPECT_IS_TREE_WITH_FIRST_CHILD_OF_ROOT_FULL_THREELEVEL_AND_SECOND_CHILD_FULL_TWOLEVEL_TREE(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_FULL_THREELEVEL_TREE(root->getChild(0)->key());

    auto secondChild = LoadInnerNode(root->getChild(1)->key());
    EXPECT_EQ(1, secondChild->numChildren());
    EXPECT_IS_FULL_TWOLEVEL_TREE(secondChild->getChild(0)->key());
  }

  void EXPECT_IS_THREELEVEL_TREE_WITH_TWO_FULL_TWOLEVEL_TREES(const Key &key) {
    auto root = LoadInnerNode(key);
    EXPECT_EQ(2, root->numChildren());
    EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(0)->key());
    EXPECT_IS_FULL_TWOLEVEL_TREE(root->getChild(1)->key());
  }
};

TEST_F(DataTreeShrinkingTest_Structure, ShrinkATwoLeafTree) {
  auto key = CreateTwoLeafTree()->key();
  Shrink(key);
  EXPECT_IS_LEAF_ONLY_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkAFourNodeThreeLeafTree) {
  auto key = CreateFourNodeThreeLeafTree();
  Shrink(key);
  EXPECT_IS_TWO_LEAF_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkATwoInnerNodeOneTwoLeavesTree) {
  auto key = CreateTwoInnerNodeOneTwoLeavesTree();
  Shrink(key);
  EXPECT_IS_TWO_INNER_NODE_TREE_WITH_ONE_LEAF_EACH(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkATwoInnerNodeTwoOneLeavesTree) {
  auto key = CreateTwoInnerNodeTwoOneLeavesTree();
  Shrink(key);
  EXPECT_IS_TWO_LEAF_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkAThreeLevelMinDataTree) {
  auto key = CreateThreeLevelMinDataTree();
  Shrink(key);
  EXPECT_IS_FULL_TWOLEVEL_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkAFourLevelMinDataTree) {
  auto key = CreateFourLevelMinDataTree();
  Shrink(key);
  EXPECT_IS_FULL_THREELEVEL_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkAFourLevelTreeWithTwoSiblingLeaves1) {
  auto key = CreateFourLevelTreeWithTwoSiblingLeaves1();
  Shrink(key);
  EXPECT_IS_FOURLEVEL_MINDATA_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkAFourLevelTreeWithTwoSiblingLeaves2) {
  auto key = CreateFourLevelTreeWithTwoSiblingLeaves2();
  Shrink(key);
  EXPECT_IS_TREE_WITH_FIRST_CHILD_OF_ROOT_FULL_THREELEVEL_AND_SECOND_CHILD_MINDATA_THREELEVEL_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkATreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel) {
  auto key = CreateTreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel();
  Shrink(key);
  EXPECT_IS_TREE_WITH_FIRST_CHILD_OF_ROOT_FULL_THREELEVEL_AND_SECOND_CHILD_FULL_TWOLEVEL_TREE(key);
}

TEST_F(DataTreeShrinkingTest_Structure, ShrinkAThreeLevelTreeWithThreeChildrenOfRoot) {
  auto key = CreateThreeLevelTreeWithThreeChildrenOfRoot();
  Shrink(key);
  EXPECT_IS_THREELEVEL_TREE_WITH_TWO_FULL_TWOLEVEL_TREES(key);
}
