#include <gtest/gtest.h>

#include "../testutils/DataTreeTest.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include "blobstore/implementations/onblocks/datatreestore/impl/algorithms.h"


using blockstore::BlockId;
using namespace blobstore::onblocks::datatreestore::algorithms;

class GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest: public DataTreeTest {
public:
  struct TestData {
    BlockId rootNode;
    BlockId expectedResult;
  };

  void check(const TestData &testData) {
    auto root = nodeStore->load(testData.rootNode).value();
    auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(nodeStore, root.get());
    EXPECT_EQ(testData.expectedResult, result->blockId());
  }

  BlockId CreateLeafOnlyTree() {
    return CreateLeaf()->blockId();
  }

  BlockId CreateTwoRightBorderNodes() {
    return CreateInner({CreateLeaf()})->blockId();
  }

  BlockId CreateThreeRightBorderNodes() {
    return CreateInner({CreateInner({CreateLeaf()})})->blockId();
  }

  TestData CreateThreeRightBorderNodes_LastFull() {
    auto node = CreateFullTwoLevel();
    auto root = CreateInner({node.get()});
    return TestData{root->blockId(), node->blockId()};
  }

  TestData CreateLargerTree() {
    auto node = CreateInner({CreateLeaf(), CreateLeaf()});
    auto root = CreateInner({CreateFullTwoLevel().get(), node.get()});
    return TestData{root->blockId(), node->blockId()};
  }

  TestData CreateThreeLevelTreeWithRightBorderSingleNodeChain() {
    auto root = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})});
    return TestData{root->blockId(), root->blockId()};
  }

  TestData CreateThreeLevelTree() {
    auto node = CreateInner({CreateLeaf(), CreateLeaf()});
    auto root = CreateInner({CreateFullTwoLevel().get(), node.get()});
    return TestData{root->blockId(), node->blockId()};
  }

  TestData CreateFullTwoLevelTree() {
    auto node = CreateFullTwoLevel();
    return TestData{node->blockId(), node->blockId()};
  }

  TestData CreateFullThreeLevelTree() {
    auto root = CreateFullThreeLevel();
    return TestData{root->blockId(), root->LastChild()->blockId()};
  }
};

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, Leaf) {
  auto leaf = nodeStore->load(CreateLeafOnlyTree()).value();
  auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(nodeStore, leaf.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, TwoRightBorderNodes) {
  auto node = nodeStore->load(CreateTwoRightBorderNodes()).value();
  auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(nodeStore, node.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, ThreeRightBorderNodes) {
  auto node = nodeStore->load(CreateThreeRightBorderNodes()).value();
  auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(nodeStore, node.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, ThreeRightBorderNodes_LastFull) {
  auto testData = CreateThreeRightBorderNodes_LastFull();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, LargerTree) {
  auto testData = CreateLargerTree();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, FullTwoLevelTree) {
  auto testData = CreateFullTwoLevelTree();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, FullThreeLevelTree) {
  auto testData = CreateFullThreeLevelTree();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, ThreeLevelTreeWithRightBorderSingleNodeChain) {
  auto testData = CreateThreeLevelTreeWithRightBorderSingleNodeChain();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, ThreeLevelTree) {
  auto testData = CreateThreeLevelTree();
  check(testData);
}
