#include "testutils/DataTreeShrinkingTest.h"

using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

class DataTreeShrinkingTest_DepthFlags: public DataTreeShrinkingTest {
public:
};

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkATwoLeafTree) {
  auto key = CreateTwoLeaf()->key();
  Shrink(key);
  CHECK_DEPTH(0, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkAFourNodeThreeLeafTree) {
  auto key = CreateFourNodeThreeLeaf()->key();
  Shrink(key);
  CHECK_DEPTH(1, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkATwoInnerNodeOneTwoLeavesTree) {
  auto key = CreateTwoInnerNodeOneTwoLeaves()->key();
  Shrink(key);
  CHECK_DEPTH(2, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkATwoInnerNodeTwoOneLeavesTree) {
  auto key = CreateTwoInnerNodeTwoOneLeaves()->key();
  Shrink(key);
  CHECK_DEPTH(1, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkAThreeLevelMinDataTree) {
  auto key = CreateThreeLevelMinData()->key();
  Shrink(key);
  CHECK_DEPTH(1, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkAFourLevelMinDataTree) {
  auto key = CreateFourLevelMinData()->key();
  Shrink(key);
  CHECK_DEPTH(2, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkAFourLevelTreeWithTwoSiblingLeaves1) {
  auto key = CreateFourLevelWithTwoSiblingLeaves1()->key();
  Shrink(key);
  CHECK_DEPTH(3, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkAFourLevelTreeWithTwoSiblingLeaves2) {
  auto key = CreateFourLevelWithTwoSiblingLeaves2()->key();
  Shrink(key);
  CHECK_DEPTH(3, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkATreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel) {
  auto key = CreateWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel()->key();
  Shrink(key);
  CHECK_DEPTH(3, key);
}

TEST_F(DataTreeShrinkingTest_DepthFlags, ShrinkAThreeLevelTreeWithThreeChildrenOfRoot) {
  auto key = CreateThreeLevelWithThreeChildrenOfRoot()->key();
  Shrink(key);
  CHECK_DEPTH(2, key);
}
