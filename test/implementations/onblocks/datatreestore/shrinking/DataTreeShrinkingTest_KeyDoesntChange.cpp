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
  auto key = CreateTwoLeaf()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourNodeThreeLeafTree) {
  auto key = CreateFourNodeThreeLeaf()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATwoInnerNodeOneTwoLeavesTree) {
  auto key = CreateTwoInnerNodeOneTwoLeaves()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATwoInnerNodeTwoOneLeavesTree) {
  auto key = CreateTwoInnerNodeTwoOneLeaves()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAThreeLevelMinDataTree) {
  auto key = CreateThreeLevelMinData()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourLevelMinDataTree) {
  auto key = CreateFourLevelMinData()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourLevelTreeWithTwoSiblingLeaves1) {
  auto key = CreateFourLevelWithTwoSiblingLeaves1()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAFourLevelTreeWithTwoSiblingLeaves2) {
  auto key = CreateFourLevelWithTwoSiblingLeaves2()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkATreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel) {
  auto key = CreateWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}

TEST_F(DataTreeShrinkingTest_KeyDoesntChange, ShrinkAThreeLevelTreeWithThreeChildrenOfRoot) {
  auto key = CreateThreeLevelWithThreeChildrenOfRoot()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_SHRINKING(key);
}
