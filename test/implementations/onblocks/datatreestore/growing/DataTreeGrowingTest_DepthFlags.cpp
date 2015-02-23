#include "testutils/DataTreeGrowingTest.h"

using blockstore::Key;

class DataTreeGrowingTest_DepthFlags: public DataTreeGrowingTest {
public:
  void CHECK_DEPTH(int depth, const Key &key) {
    auto node = LoadInnerNode(key);
    EXPECT_EQ(depth, node->depth());
    if (depth > 1) {
      for (int i = 0; i < node->numChildren(); ++i) {
        CHECK_DEPTH(depth-1, node->getChild(i)->key());
      }
    }
  }
};

TEST_F(DataTreeGrowingTest_DepthFlags, GrowAOneNodeTree) {
  auto key = CreateTreeAddOneLeafReturnRootKey();
  CHECK_DEPTH(1, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowATwoNodeTree) {
  auto key = CreateTreeAddTwoLeavesReturnRootKey();
  CHECK_DEPTH(1, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowATwoLevelThreeNodeTree) {
  auto key = CreateTreeAddThreeLeavesReturnRootKey();
  CHECK_DEPTH(1, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowAThreeNodeChainedTree) {
  auto key = CreateThreeNodeChainedTreeReturnRootKey();
  AddLeafTo(key);
  CHECK_DEPTH(2, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowAFullTwoLevelTree) {
  auto key = CreateFullTwoLevel()->key();
  AddLeafTo(key);
  CHECK_DEPTH(2, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowAThreeLevelTreeWithLowerLevelFull) {
  auto key = CreateThreeLevelTreeWithLowerLevelFullReturnRootKey();
  AddLeafTo(key);
  CHECK_DEPTH(2, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowAFullThreeLevelTree) {
  auto key = CreateFullThreeLevel()->key();
  AddLeafTo(key);
  CHECK_DEPTH(3, key);
}

TEST_F(DataTreeGrowingTest_DepthFlags, GrowAThreeLevelTreeWithTwoFullSubtrees) {
  auto key = CreateThreeLevelTreeWithTwoFullSubtrees();
  AddLeafTo(key);
  CHECK_DEPTH(2, key);
}
