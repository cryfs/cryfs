#include "../testutils/DataTreeTest.h"
#include <google/gmock/gmock.h>

using ::testing::_;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

class TraversorMock {
public:
  MOCK_METHOD2(called, void(DataLeafNode*, uint32_t));
};

MATCHER_P(KeyEq, expected, "node key equals") {
  return arg->key() == expected;
}

class DataTreeTest_TraverseLeaves: public DataTreeTest {
public:
  void TraverseLeaves(DataNode *root, uint32_t beginIndex, uint32_t endIndex) {
    root->flush();
    auto tree = treeStore.load(root->key());
    tree->traverseLeaves(beginIndex, endIndex, [this] (DataLeafNode *leaf, uint32_t nodeIndex) {
      traversor.called(leaf, nodeIndex);
    });
  }
  TraversorMock traversor;
};

TEST_F(DataTreeTest_TraverseLeaves, TraverseSingleLeafTree) {
  auto root = CreateLeaf();
  EXPECT_CALL(traversor, called(KeyEq(root->key()), 0)).Times(1);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInSingleLeafTree1) {
  auto root = CreateLeaf();
  EXPECT_CALL(traversor, called(_, _)).Times(0);

  TraverseLeaves(root.get(), 0, 0);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInSingleLeafTree2) {
  auto root = CreateLeaf();
  EXPECT_CALL(traversor, called(_, _)).Times(0);

  TraverseLeaves(root.get(), 1, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  auto firstleafkey = root->getChild(0)->key();
  EXPECT_CALL(traversor, called(KeyEq(firstleafkey), 0)).Times(1);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  auto leafkey = root->getChild(5)->key();
  EXPECT_CALL(traversor, called(KeyEq(leafkey), 5)).Times(1);

  TraverseLeaves(root.get(), 5, 6);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  auto leafkey = root->getChild(DataInnerNode::MAX_STORED_CHILDREN-1)->key();
  EXPECT_CALL(traversor, called(KeyEq(leafkey), DataInnerNode::MAX_STORED_CHILDREN-1)).Times(1);

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN-1, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInFullTwolevelTree1) {
  auto root = CreateFullTwoLevel();
  EXPECT_CALL(traversor, called(_, _)).Times(0);

  TraverseLeaves(root.get(), 0, 0);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInFullTwolevelTree2) {
  auto root = CreateFullTwoLevel();
  EXPECT_CALL(traversor,called(_, _)).Times(0);

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto firstleafkey = LoadInnerNode(root->getChild(0)->key())->getChild(0)->key();
  EXPECT_CALL(traversor, called(KeyEq(firstleafkey), 0)).Times(1);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto leafkey = LoadInnerNode(root->getChild(0)->key())->getChild(5)->key();
  EXPECT_CALL(traversor, called(KeyEq(leafkey), 5)).Times(1);

  TraverseLeaves(root.get(), 5, 6);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto leafkey = LoadInnerNode(root->getChild(1)->key())->getChild(0)->key();
  EXPECT_CALL(traversor, called(KeyEq(leafkey), DataInnerNode::MAX_STORED_CHILDREN)).Times(1);

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN, DataInnerNode::MAX_STORED_CHILDREN+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (int i = 0; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    auto leafkey = root->getChild(i)->key();
    EXPECT_CALL(traversor, called(KeyEq(leafkey), i)).Times(1);
  }

  TraverseLeaves(root.get(), 0, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 0; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    auto leafkey = node->getChild(i)->key();
    EXPECT_CALL(traversor, called(KeyEq(leafkey), i)).Times(1);
  }
  Key lastleafkey = LoadInnerNode(root->getChild(1)->key())->getChild(0)->key();
  EXPECT_CALL(traversor, called(KeyEq(lastleafkey), DataInnerNode::MAX_STORED_CHILDREN)).Times(1);

  TraverseLeaves(root.get(), 0, DataInnerNode::MAX_STORED_CHILDREN+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstChildOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 0; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    auto leafkey = node->getChild(i)->key();
    EXPECT_CALL(traversor, called(KeyEq(leafkey), i)).Times(1);
  }

  TraverseLeaves(root.get(), 0, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 0; i < 5; ++i) {
    auto leafkey = node->getChild(i)->key();
    EXPECT_CALL(traversor, called(KeyEq(leafkey), i)).Times(1);
  }

  TraverseLeaves(root.get(), 0, 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseInnerPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 5; i < 10; ++i) {
    auto leafkey = node->getChild(i)->key();
    EXPECT_CALL(traversor, called(KeyEq(leafkey), i)).Times(1);
  }

  TraverseLeaves(root.get(), 5, 10);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 5; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    auto leafkey = node->getChild(i)->key();
    EXPECT_CALL(traversor, called(KeyEq(leafkey), i)).Times(1);
  }
  Key lastleafkey = LoadInnerNode(root->getChild(1)->key())->getChild(0)->key();
  EXPECT_CALL(traversor, called(KeyEq(lastleafkey), DataInnerNode::MAX_STORED_CHILDREN)).Times(1);

  TraverseLeaves(root.get(), 5, DataInnerNode::MAX_STORED_CHILDREN+1);
}

//TODO First/Inner/LastPart of FullTwoLevelTree
//TODO Test cases with a larger threelevel tree (say 5 children being full twolevel trees)
//TODO Some few testcases with full threelevel tree
//TODO Some few testcases with fourlevel mindata tree

//TODO ...more test cases?
