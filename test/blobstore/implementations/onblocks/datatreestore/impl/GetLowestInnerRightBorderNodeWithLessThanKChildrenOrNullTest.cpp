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

class GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest: public DataTreeTest {
public:
  struct TestData {
    Key rootNode;
    Key expectedResult;
  };

  void check(const TestData &testData) {
    auto root = nodeStore->load(testData.rootNode).value();
    auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(nodeStore, root.get());
    EXPECT_EQ(testData.expectedResult, result->key());
  }

  TestData CreateTwoRightBorderNodes() {
    auto node = CreateInner({CreateLeaf()});
    return TestData{node->key(), node->key()};
  }

  TestData CreateThreeRightBorderNodes() {
    auto node = CreateInner({CreateLeaf()});
    auto root = CreateInner({node.get()});
    return TestData{root->key(), node->key()};
  }

  TestData CreateThreeRightBorderNodes_LastFull() {
    auto root = CreateInner({CreateFullTwoLevel()});
    return TestData{root->key(), root->key()};
  }

  TestData CreateLargerTree() {
    auto node = CreateInner({CreateLeaf(), CreateLeaf()});
    auto root = CreateInner({CreateFullTwoLevel().get(), node.get()});
    return TestData{root->key(), node->key()};
  }
};

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, Leaf) {
  auto leaf = nodeStore->createNewLeafNode();
  auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(nodeStore, leaf.get());
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
  auto root = CreateFullTwoLevel();
  auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(nodeStore, root.get());
  EXPECT_EQ(nullptr, result.get());
}

TEST_F(GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNullTest, FullThreeLevelTree) {
  auto root = CreateFullThreeLevel();
  auto result = GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(nodeStore, root.get());
  EXPECT_EQ(nullptr, result.get());
}
