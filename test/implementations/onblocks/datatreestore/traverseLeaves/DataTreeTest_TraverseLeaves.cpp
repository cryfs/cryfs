#include "../testutils/DataTreeTest.h"
#include <google/gmock/gmock.h>

using ::testing::_;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

using std::unique_ptr;

class TraversorMock {
public:
  MOCK_METHOD2(called, void(DataLeafNode*, uint32_t));
};

MATCHER_P(KeyEq, expected, "node key equals") {
  return arg->key() == expected;
}

class DataTreeTest_TraverseLeaves: public DataTreeTest {
public:
  unique_ptr<DataInnerNode> CreateThreeLevel() {
    return CreateInner({
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateInner({CreateLeaf(), CreateLeaf(), CreateLeaf()})});
  }

  unique_ptr<DataInnerNode> CreateFourLevel() {
    return CreateInner({
      CreateFullThreeLevel(),
      CreateFullThreeLevel(),
      CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})})
    });
  }

  void EXPECT_TRAVERSE_LEAF(const Key &key, uint32_t leafIndex) {
    EXPECT_CALL(traversor, called(KeyEq(key), leafIndex)).Times(1);
  }

  void EXPECT_TRAVERSE_ALL_CHILDREN_OF(const DataInnerNode &node, uint32_t firstLeafIndex) {
    for (int i = 0; i < node.numChildren(); ++i) {
      EXPECT_TRAVERSE_LEAF(node.getChild(i)->key(), firstLeafIndex+i);
    }
  }

  void EXPECT_DONT_TRAVERSE_ANY_LEAVES() {
    EXPECT_CALL(traversor, called(_, _)).Times(0);
  }

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
  EXPECT_TRAVERSE_LEAF(root->key(), 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInSingleLeafTree1) {
  auto root = CreateLeaf();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(root.get(), 0, 0);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInSingleLeafTree2) {
  auto root = CreateLeaf();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(root.get(), 1, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->getChild(0)->key(), 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->getChild(5)->key(), 5);

  TraverseLeaves(root.get(), 5, 6);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->getChild(DataInnerNode::MAX_STORED_CHILDREN-1)->key(), DataInnerNode::MAX_STORED_CHILDREN-1);

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN-1, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInFullTwolevelTree1) {
  auto root = CreateFullTwoLevel();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(root.get(), 0, 0);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInFullTwolevelTree2) {
  auto root = CreateFullTwoLevel();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(0)->key())->getChild(0)->key(), 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(0)->key())->getChild(5)->key(), 5);

  TraverseLeaves(root.get(), 5, 6);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(1)->key())->getChild(0)->key(), DataInnerNode::MAX_STORED_CHILDREN);

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN, DataInnerNode::MAX_STORED_CHILDREN+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*root, 0);

  TraverseLeaves(root.get(), 0, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(0)->key()), 0);
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(1)->key())->getChild(0)->key(), DataInnerNode::MAX_STORED_CHILDREN);

  TraverseLeaves(root.get(), 0, DataInnerNode::MAX_STORED_CHILDREN+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstChildOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(0)->key()), 0);

  TraverseLeaves(root.get(), 0, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(root->getChild(i)->key(), i);
  }

  TraverseLeaves(root.get(), 0, 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseInnerPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (int i = 5; i < 10; ++i) {
    EXPECT_TRAVERSE_LEAF(root->getChild(i)->key(), i);
  }

  TraverseLeaves(root.get(), 5, 10);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (int i = 5; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    EXPECT_TRAVERSE_LEAF(root->getChild(i)->key(), i);
  }

  TraverseLeaves(root.get(), 5, DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(node->getChild(i)->key(), i);
  }

  TraverseLeaves(root.get(), 0, 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseInnerPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 5; i < 10; ++i) {
    EXPECT_TRAVERSE_LEAF(node->getChild(i)->key(), i);
  }

  TraverseLeaves(root.get(), 5, 10);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->key());
  for (int i = 5; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    EXPECT_TRAVERSE_LEAF(node->getChild(i)->key(), i);
  }
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(1)->key())->getChild(0)->key(), DataInnerNode::MAX_STORED_CHILDREN);

  TraverseLeaves(root.get(), 5, DataInnerNode::MAX_STORED_CHILDREN+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(0)->key())->getChild(0)->key(), 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  uint32_t numLeaves = DataInnerNode::MAX_STORED_CHILDREN * 5 + 3;
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->LastChild()->key())->LastChild()->key(), numLeaves-1);

  TraverseLeaves(root.get(), numLeaves-1, numLeaves);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  uint32_t wantedLeafIndex = DataInnerNode::MAX_STORED_CHILDREN * 2 + 5;
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(2)->key())->getChild(5)->key(), wantedLeafIndex);

  TraverseLeaves(root.get(), wantedLeafIndex, wantedLeafIndex+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse all leaves in the first two children of the root
  for(int i = 0; i < 2; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->key()), i * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse some of the leaves in the third child of the root
  auto child = LoadInnerNode(root->getChild(2)->key());
  for(int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), 2 * DataInnerNode::MAX_STORED_CHILDREN + i);
  }

  TraverseLeaves(root.get(), 0, 2 * DataInnerNode::MAX_STORED_CHILDREN + 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddlePartOfThreelevelTree_OnlyFullChildren) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->getChild(1)->key());
  for(int i = 5; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), DataInnerNode::MAX_STORED_CHILDREN + i);
  }
  //Traverse all leaves in the third and fourth child of the root
  for(int i = 2; i < 4; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->key()), i * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse some of the leaves in the fifth child of the root
  child = LoadInnerNode(root->getChild(4)->key());
  for(int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), 4 * DataInnerNode::MAX_STORED_CHILDREN + i);
  }

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN + 5, 4 * DataInnerNode::MAX_STORED_CHILDREN + 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddlePartOfThreelevelTree_AlsoLastNonfullChild) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->getChild(1)->key());
  for(int i = 5; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), DataInnerNode::MAX_STORED_CHILDREN + i);
  }
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(int i = 2; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->key()), i * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse some of the leaves in the sixth child of the root
  child = LoadInnerNode(root->getChild(5)->key());
  for(int i = 0; i < 2; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), 5 * DataInnerNode::MAX_STORED_CHILDREN + i);
  }

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN + 5, 5 * DataInnerNode::MAX_STORED_CHILDREN + 2);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->getChild(1)->key());
  for(int i = 5; i < DataInnerNode::MAX_STORED_CHILDREN; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), DataInnerNode::MAX_STORED_CHILDREN + i);
  }
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(int i = 2; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->key()), i * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse all of the leaves in the sixth child of the root
  child = LoadInnerNode(root->getChild(5)->key());
  for(int i = 0; i < child->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), 5 * DataInnerNode::MAX_STORED_CHILDREN + i);
  }

  TraverseLeaves(root.get(), DataInnerNode::MAX_STORED_CHILDREN + 5, 5 * DataInnerNode::MAX_STORED_CHILDREN + child->numChildren());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->key()), i * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse all of the leaves in the sixth child of the root
  auto child = LoadInnerNode(root->getChild(5)->key());
  for(int i = 0; i < child->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->key(), 5 * DataInnerNode::MAX_STORED_CHILDREN + i);
  }

  TraverseLeaves(root.get(), 0, 5 * DataInnerNode::MAX_STORED_CHILDREN + child->numChildren());
}

//Disabled because it takes too long
TEST_F(DataTreeTest_TraverseLeaves, DISABLED_TraverseAllLeavesOfFourLevelTree) {
  auto root = CreateFourLevel();
  //Traverse all leaves of the full threelevel tree in the first child
  auto firstChild = LoadInnerNode(root->getChild(0)->key());
  for(int i = 0; i < firstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(firstChild->getChild(i)->key()), i * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse all leaves of the full threelevel tree in the second child
  auto secondChild = LoadInnerNode(root->getChild(1)->key());
  for(int i = 0; i < secondChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(secondChild->getChild(i)->key()), (DataInnerNode::MAX_STORED_CHILDREN + i) * DataInnerNode::MAX_STORED_CHILDREN);
  }
  //Traverse all leaves of the non-full threelevel tree in the third child
  auto thirdChild = LoadInnerNode(root->getChild(2)->key());
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(thirdChild->getChild(0)->key()), 2 * DataInnerNode::MAX_STORED_CHILDREN * DataInnerNode::MAX_STORED_CHILDREN);
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(thirdChild->getChild(1)->key())->getChild(0)->key(), 2 * DataInnerNode::MAX_STORED_CHILDREN * DataInnerNode::MAX_STORED_CHILDREN + DataInnerNode::MAX_STORED_CHILDREN);

  TraverseLeaves(root.get(), 0, 2*DataInnerNode::MAX_STORED_CHILDREN*DataInnerNode::MAX_STORED_CHILDREN + DataInnerNode::MAX_STORED_CHILDREN + 1);
}

//TODO Traverse inner part of four level tree
//TODO Refactor the test cases that are too long
