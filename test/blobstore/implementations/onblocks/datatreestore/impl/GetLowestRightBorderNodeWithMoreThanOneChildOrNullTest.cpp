#include <gtest/gtest.h>

#include "../testutils/DataTreeTest.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include "blobstore/implementations/onblocks/datatreestore/impl/algorithms.h"

using ::testing::Test;
using std::pair;
using std::make_pair;

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using namespace blobstore::onblocks::datatreestore::algorithms;

class GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest: public DataTreeTest {
public:
  struct TestData {
    Key rootNode;
    Key expectedResult;
  };

  void check(const TestData &testData) {
    auto root = nodeStore->load(testData.rootNode).value();
    auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(nodeStore, root.get());
    EXPECT_EQ(testData.expectedResult, result->key());
  }

  Key CreateLeafOnlyTree() {
    return CreateLeaf()->key();
  }

  Key CreateTwoRightBorderNodes() {
    return CreateInner({CreateLeaf()})->key();
  }

  Key CreateThreeRightBorderNodes() {
    return CreateInner({CreateInner({CreateLeaf()})})->key();
  }

  TestData CreateThreeRightBorderNodes_LastFull() {
    auto node = CreateFullTwoLevel();
    auto root = CreateInner({node.get()});
    return TestData{root->key(), node->key()};
  }

  TestData CreateLargerTree() {
    auto node = CreateInner({CreateLeaf(), CreateLeaf()});
    auto root = CreateInner({CreateFullTwoLevel().get(), node.get()});
    return TestData{root->key(), node->key()};
  }

  TestData CreateThreeLevelTreeWithRightBorderSingleNodeChain() {
    auto root = CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})});
    return TestData{root->key(), root->key()};
  }

  TestData CreateThreeLevelTree() {
    auto node = CreateInner({CreateLeaf(), CreateLeaf()});
    auto root = CreateInner({CreateFullTwoLevel().get(), node.get()});
    return TestData{root->key(), node->key()};
  }

  TestData CreateFullTwoLevelTree() {
    auto node = CreateFullTwoLevel();
    return TestData{node->key(), node->key()};
  }

  TestData CreateFullThreeLevelTree() {
    auto root = CreateFullThreeLevel();
    return TestData{root->key(), root->LastChild()->key()};
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
