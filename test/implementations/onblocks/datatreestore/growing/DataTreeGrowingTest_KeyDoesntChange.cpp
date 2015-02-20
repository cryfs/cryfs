#include "testutils/DataTreeGrowingTest.h"

class DataTreeGrowingTest_KeyDoesntChange: public DataTreeGrowingTest {};

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowAOneNodeTree) {
  auto key = CreateLeafOnlyTree()->key();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowATwoNodeTree) {
  auto key = CreateTreeAddOneLeafReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowATwoLevelThreeNodeTree) {
  auto key = CreateTreeAddTwoLeavesReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowAThreeNodeChainedTree) {
  auto root_key = CreateThreeNodeChainedTreeReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowAThreeLevelTreeWithLowerLevelFull) {
  auto root_key = CreateThreeLevelTreeWithLowerLevelFullReturnRootKey();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowAFullTwoLevelTree) {
  auto root_key = CreateFullTwoLevelTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowAFullThreeLevelTree) {
  auto root_key = CreateFullThreeLevelTree();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}

TEST_F(DataTreeGrowingTest_KeyDoesntChange, GrowAThreeLevelTreeWithTwoFullSubtrees) {
  auto root_key = CreateThreeLevelTreeWithTwoFullSubtrees();
  EXPECT_KEY_DOESNT_CHANGE_WHEN_GROWING(root_key);
}
