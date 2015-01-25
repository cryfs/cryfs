#include "gtest/gtest.h"

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"

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
using blobstore::onblocks::datatreestore::impl::GetLowestRightBorderNodeWithLessThanKChildrenOrNull;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

class GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest: public Test {
public:
  GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest():
    nodeStore(make_unique<FakeBlockStore>()) {
  }

  struct TestData {
    TestData(Key rootNode_, Key expectedResult_): rootNode(rootNode_), expectedResult(expectedResult_) {}
    Key rootNode;
    Key expectedResult;
  };

  void check(const TestData &testData) {
    auto root = nodeStore.load(testData.rootNode);
    auto result = GetLowestRightBorderNodeWithLessThanKChildrenOrNull::run(&nodeStore, root.get());
    EXPECT_EQ(testData.expectedResult, result->key());
  }

  void FillNode(DataInnerNode *node) {
    for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
      node->addChild(*nodeStore.createNewLeafNode());
    }
  }

  void FillNodeTwoLevel(DataInnerNode *node) {
    for(unsigned int i=node->numChildren(); i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
      auto inner_node = nodeStore.createNewInnerNode(*nodeStore.createNewLeafNode());
      for(unsigned int j = 1;j < DataInnerNode::MAX_STORED_CHILDREN; ++j) {
        inner_node->addChild(*nodeStore.createNewLeafNode());
      }
      node->addChild(*inner_node);
    }
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

  Key CreateFullTwoLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto root = nodeStore.createNewInnerNode(*leaf);
    FillNode(root.get());
    return root->key();
  }

  Key CreateFullThreeLevelTree() {
    auto leaf = nodeStore.createNewLeafNode();
    auto node = nodeStore.createNewInnerNode(*leaf);
    auto root = nodeStore.createNewInnerNode(*node);
    FillNode(node.get());
    FillNodeTwoLevel(root.get());
    return root->key();
  }

  DataNodeStore nodeStore;
};

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, Leaf) {
  auto leaf = nodeStore.createNewLeafNode();
  auto result = GetLowestRightBorderNodeWithLessThanKChildrenOrNull::run(&nodeStore, leaf.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, TwoRightBorderNodes) {
  auto testData = CreateTwoRightBorderNodes();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, ThreeRightBorderNodes) {
  auto testData = CreateThreeRightBorderNodes();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, ThreeRightBorderNodes_LastFull) {
  auto testData = CreateThreeRightBorderNodes_LastFull();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, LargerTree) {
  auto testData = CreateLargerTree();
  check(testData);
}

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, FullTwoLevelTree) {
  auto root = nodeStore.load(CreateFullTwoLevelTree());
  auto result = GetLowestRightBorderNodeWithLessThanKChildrenOrNull::run(&nodeStore, root.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestRightBorderNodeWithLessThanKChildrenOrNullTest, FullThreeLevelTree) {
  auto root = nodeStore.load(CreateFullThreeLevelTree());
  auto result = GetLowestRightBorderNodeWithLessThanKChildrenOrNull::run(&nodeStore, root.get());
  EXPECT_EQ(nullptr, result.get());
}

}
}
}
