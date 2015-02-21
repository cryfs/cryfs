#include "google/gtest/gtest.h"

#include "../testutils/DataTreeTest.h"
#include "../../../../../implementations/onblocks/datatreestore/DataTree.h"
#include "../../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../../implementations/onblocks/datanodestore/DataInnerNode.h"
#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"
#include "../../../../../implementations/onblocks/datatreestore/impl/algorithms.h"

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;
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
    TestData(Key rootNode_, Key expectedResult_): rootNode(rootNode_), expectedResult(expectedResult_) {}
    Key rootNode;
    Key expectedResult;
  };

  void check(const TestData &testData) {
    auto root = nodeStore.load(testData.rootNode);
    auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(&nodeStore, root.get());
    EXPECT_EQ(testData.expectedResult, result->key());
  }

  Key CreateLeafOnlyTree() {
    return nodeStore.createNewLeafNode()->key();
  }

  Key CreateTwoRightBorderNodes() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    return node->key(), node->key();
  }

  Key CreateThreeRightBorderNodes() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    auto root = nodeStore.createNewInnerNode(*node);
    return root->key();
  }

  TestData CreateThreeRightBorderNodes_LastFull() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    FillNode(node.get());
    auto root = nodeStore.createNewInnerNode(*node);
    return TestData(root->key(), node->key());
  }

  TestData CreateLargerTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto leaf2 = nodeStore.createNewLeafNode();
    auto leaf3 = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    FillNode(node.get());
    auto node2 = nodeStore.createNewInnerNode(*leaf2);
    node2->addChild(*leaf3);
    auto root = nodeStore.createNewInnerNode(*node);
    root->addChild(*node2);
    return TestData(root->key(), node2->key());
  }

  TestData CreateThreeLevelTreeWithRightBorderSingleNodeChain() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node1 = nodeStore.createNewInnerNode(*leaf);
    FillNode(node1.get());
    auto leaf2 = nodeStore.createNewLeafNode();
    auto node2 = nodeStore.createNewInnerNode(*leaf2);
    auto root = nodeStore.createNewInnerNode(*node1);
    root->addChild(*node2);
    return TestData(root->key(), root->key());
  }

  TestData CreateThreeLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node1 = nodeStore.createNewInnerNode(*leaf);
    FillNode(node1.get());
    auto leaf2 = nodeStore.createNewLeafNode();
    auto leaf3 = nodeStore.createNewLeafNode();
    auto node2 = nodeStore.createNewInnerNode(*leaf2);
    node2->addChild(*leaf3);
    auto root = nodeStore.createNewInnerNode(*node1);
    root->addChild(*node2);
    return TestData(root->key(), node2->key());
  }

  TestData CreateFullTwoLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    FillNode(node.get());
    return TestData(node->key(), node->key());
  }

  TestData CreateFullThreeLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto firstFullTwoLevelTree = nodeStore.createNewInnerNode(*leaf);
    FillNode(firstFullTwoLevelTree.get());
    auto root = nodeStore.createNewInnerNode(*firstFullTwoLevelTree);
    for (int i = 1; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
      auto leaf2 = nodeStore.createNewLeafNode();
      auto fullTwoLevelTree = nodeStore.createNewInnerNode(*leaf2);
      FillNode(fullTwoLevelTree.get());
      root->addChild(*fullTwoLevelTree);
    }
    return TestData(root->key(), root->LastChild()->key());
  }
};

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, Leaf) {
  auto leaf = nodeStore.load(CreateLeafOnlyTree());
  auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(&nodeStore, leaf.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, TwoRightBorderNodes) {
  auto node = nodeStore.load(CreateTwoRightBorderNodes());
  auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(&nodeStore, node.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithMoreThanOneChildOrNullTest, ThreeRightBorderNodes) {
  auto node = nodeStore.load(CreateThreeRightBorderNodes());
  auto result = GetLowestRightBorderNodeWithMoreThanOneChildOrNull(&nodeStore, node.get());
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
