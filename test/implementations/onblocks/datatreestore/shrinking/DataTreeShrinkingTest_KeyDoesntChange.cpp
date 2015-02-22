#include "testutils/DataTreeShrinkingTest.h"

using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

class DataTreeShrinkingTest_KeyDoesntChange: public DataTreeShrinkingTest {
public:
  void EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(const Key &key) {
    DataTree tree(&nodeStore, nodeStore.load(key));
    tree.removeLastDataLeaf();
    EXPECT_EQ(key, tree.key());
  }
};

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATwoLeafTree) {
  auto key = CreateTwoLeafTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourNodeThreeLeafTree) {
  auto key = CreateFourNodeThreeLeafTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATwoInnerNodeOneTwoLeavesTree) {
  auto key = CreateTwoInnerNodeOneTwoLeavesTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATwoInnerNodeTwoOneLeavesTree) {
  auto key = CreateTwoInnerNodeTwoOneLeavesTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAThreeLevelMinDataTree) {
  auto key = CreateThreeLevelMinDataTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourLevelMinDataTree) {
  auto key = CreateFourLevelMinDataTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourLevelTreeWithTwoSiblingLeaves1) {
  auto key = CreateFourLevelTreeWithTwoSiblingLeaves1();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourLevelTreeWithTwoSiblingLeaves2) {
  auto key = CreateFourLevelTreeWithTwoSiblingLeaves2();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel) {
  auto key = CreateTreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAThreeLevelTreeWithThreeChildrenOfRoot) {
  auto key = CreateThreeLevelTreeWithThreeChildrenOfRoot();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}
