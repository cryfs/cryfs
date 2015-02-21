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

class GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest: public DataTreeTest {
public:
  struct TestData {
    TestData(Key rootNode_, Key expectedResult_): rootNode(rootNode_), expectedResult(expectedResult_) {}
    Key rootNode;
    Key expectedResult;
  };

  void check(const TestData &testData) {
    auto root = nodeStore.load(testData.rootNode);
    auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(&nodeStore, root.get());
    EXPECT_EQ(testData.expectedResult, result->key());
  }

  TestData CreateTwoRightBorderNodes() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    return TestData(node->key(), node->key());
  }

  TestData CreateThreeRightBorderNodes() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    auto root = nodeStore.createNewInnerNode(*node);
    return TestData(root->key(), node->key());
  }

  TestData CreateThreeRightBorderNodes_LastFull() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    FillNode(node.get());
    auto root = nodeStore.createNewInnerNode(*node);
    return TestData(root->key(), root->key());
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
};

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, Leaf) {
  auto leaf = nodeStore.createNewLeafNode();
  auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(&nodeStore, leaf.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, TwoRightBorderNodes) {
  auto testData = CreateTwoRightBorderNodes();
  check(testData);
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, ThreeRightBorderNodes) {
  auto testData = CreateThreeRightBorderNodes();
  check(testData);
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, ThreeRightBorderNodes_LastFull) {
  auto testData = CreateThreeRightBorderNodes_LastFull();
  check(testData);
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, LargerTree) {
  auto testData = CreateLargerTree();
  check(testData);
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, FullTwoLevelTree) {
  auto root = nodeStore.load(CreateFullTwoLevelTree());
  auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(&nodeStore, root.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, FullThreeLevelTree) {
  auto root = nodeStore.load(CreateFullThreeLevelTree());
  auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(&nodeStore, root.get());
  EXPECT_EQ(nullptr, result.get());
}
