#include "testutils/DataTreeTest.h"
#include <gmock/gmock.h>

using ::testing::_;
using ::testing::Invoke;
using ::testing::Eq;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datatreestore::LeafHandle;
using blockstore::BlockId;

using cpputils::unique_ref;
using cpputils::Data;
using std::shared_ptr;
using std::make_shared;

class TraversorMock {
public:
  MOCK_METHOD3(calledExistingLeaf, void(DataLeafNode*, bool, uint32_t));
  MOCK_METHOD1(calledCreateLeaf, shared_ptr<Data>(uint32_t));
};

MATCHER_P(KeyEq, expected, "node blockId equals") {
  return arg->blockId() == expected;
}

class DataTreeTest_TraverseLeaves: public DataTreeTest {
public:
  DataTreeTest_TraverseLeaves() :traversor() {}

  unique_ref<DataInnerNode> CreateThreeLevel() {
    return CreateInner({
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateFullTwoLevel(),
      CreateInner({CreateLeaf(), CreateLeaf(), CreateLeaf()})});
  }

  unique_ref<DataInnerNode> CreateFourLevel() {
    return CreateInner({
      CreateFullThreeLevel(),
      CreateFullThreeLevel(),
      CreateInner({CreateFullTwoLevel(), CreateInner({CreateLeaf()})})
    });
  }

  void EXPECT_CREATE_LEAF(uint32_t leafIndex) {
    uint64_t maxBytesPerLeaf = nodeStore->layout().maxBytesPerLeaf();
    EXPECT_CALL(traversor, calledCreateLeaf(Eq(leafIndex))).Times(1).WillOnce(Invoke([maxBytesPerLeaf] (uint32_t) {
        return make_shared<Data>(maxBytesPerLeaf);
    }));
  }

  void EXPECT_TRAVERSE_LEAF(const BlockId &blockId, bool isRightBorderLeaf, uint32_t leafIndex) {
    EXPECT_CALL(traversor, calledExistingLeaf(KeyEq(blockId), isRightBorderLeaf, leafIndex)).Times(1);
  }

  void EXPECT_TRAVERSE_ALL_CHILDREN_OF(const DataInnerNode &node, bool isRightBorderNode, uint32_t firstLeafIndex) {
    for (unsigned int i = 0; i < node.numChildren(); ++i) {
      EXPECT_TRAVERSE_LEAF(node.getChild(i)->blockId(), isRightBorderNode && i == node.numChildren()-1, firstLeafIndex+i);
    }
  }

  void EXPECT_DONT_TRAVERSE_ANY_LEAVES() {
    EXPECT_CALL(traversor, calledExistingLeaf(_, _, _)).Times(0);
    EXPECT_CALL(traversor, calledCreateLeaf(_)).Times(0);
  }

  void TraverseLeaves(DataNode *root, uint32_t beginIndex, uint32_t endIndex) {
    root->flush();
    auto tree = treeStore.load(root->blockId()).value();
    tree->traverseLeaves(beginIndex, endIndex, [this] (uint32_t nodeIndex, bool isRightBorderNode,LeafHandle leaf) {
      traversor.calledExistingLeaf(leaf.node(), isRightBorderNode,  nodeIndex);
    }, [this] (uint32_t nodeIndex) -> Data {
        return traversor.calledCreateLeaf(nodeIndex)->copy();
    });
  }

  TraversorMock traversor;
};

TEST_F(DataTreeTest_TraverseLeaves, TraverseSingleLeafTree) {
  auto root = CreateLeaf();
  EXPECT_TRAVERSE_LEAF(root->blockId(), true, 0);

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
  EXPECT_TRAVERSE_LEAF(root->getChild(0)->blockId(), false, 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->getChild(5)->blockId(), false, 5);

  TraverseLeaves(root.get(), 5, 6);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->getChild(nodeStore->layout().maxChildrenPerInnerNode()-1)->blockId(), true, nodeStore->layout().maxChildrenPerInnerNode()-1);

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode()-1, nodeStore->layout().maxChildrenPerInnerNode());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInFullTwolevelTree1) {
  auto root = CreateFullTwoLevel();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(root.get(), 0, 0);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseNothingInFullTwolevelTree2) {
  auto root = CreateFullTwoLevel();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode(), nodeStore->layout().maxChildrenPerInnerNode());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(0)->blockId())->getChild(0)->blockId(), false, 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(0)->blockId())->getChild(5)->blockId(), false, 5);

  TraverseLeaves(root.get(), 5, 6);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(1)->blockId())->getChild(0)->blockId(), true, nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode(), nodeStore->layout().maxChildrenPerInnerNode()+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*root, true, 0);

  TraverseLeaves(root.get(), 0, nodeStore->layout().maxChildrenPerInnerNode());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(0)->blockId()), false, 0);
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(1)->blockId())->getChild(0)->blockId(), true, nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(root.get(), 0, nodeStore->layout().maxChildrenPerInnerNode()+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstChildOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(0)->blockId()), false, 0);

  TraverseLeaves(root.get(), 0, nodeStore->layout().maxChildrenPerInnerNode());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(root->getChild(i)->blockId(), false, i);
  }

  TraverseLeaves(root.get(), 0, 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseInnerPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (unsigned int i = 5; i < 10; ++i) {
    EXPECT_TRAVERSE_LEAF(root->getChild(i)->blockId(), false, i);
  }

  TraverseLeaves(root.get(), 5, 10);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(root->getChild(i)->blockId(), i==nodeStore->layout().maxChildrenPerInnerNode()-1, i);
  }

  TraverseLeaves(root.get(), 5, nodeStore->layout().maxChildrenPerInnerNode());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->blockId());
  for (unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(node->getChild(i)->blockId(), false, i);
  }

  TraverseLeaves(root.get(), 0, 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseInnerPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->blockId());
  for (unsigned int i = 5; i < 10; ++i) {
    EXPECT_TRAVERSE_LEAF(node->getChild(i)->blockId(), false, i);
  }

  TraverseLeaves(root.get(), 5, 10);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->getChild(0)->blockId());
  for (unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(node->getChild(i)->blockId(), false, i);
  }
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(1)->blockId())->getChild(0)->blockId(), true, nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(root.get(), 5, nodeStore->layout().maxChildrenPerInnerNode()+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(0)->blockId())->getChild(0)->blockId(), false, 0);

  TraverseLeaves(root.get(), 0, 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  uint32_t numLeaves = nodeStore->layout().maxChildrenPerInnerNode() * 5 + 3;
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->LastChild()->blockId())->LastChild()->blockId(), true, numLeaves-1);

  TraverseLeaves(root.get(), numLeaves-1, numLeaves);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddleLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  uint32_t wantedLeafIndex = nodeStore->layout().maxChildrenPerInnerNode() * 2 + 5;
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->getChild(2)->blockId())->getChild(5)->blockId(), false, wantedLeafIndex);

  TraverseLeaves(root.get(), wantedLeafIndex, wantedLeafIndex+1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseFirstPartOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse all leaves in the first two children of the root
  for(unsigned int i = 0; i < 2; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some of the leaves in the third child of the root
  auto child = LoadInnerNode(root->getChild(2)->blockId());
  for(unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), false, 2 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(root.get(), 0, 2 * nodeStore->layout().maxChildrenPerInnerNode() + 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddlePartOfThreelevelTree_OnlyFullChildren) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->getChild(1)->blockId());
  for(unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), false, nodeStore->layout().maxChildrenPerInnerNode() + i);
  }
  //Traverse all leaves in the third and fourth child of the root
  for(unsigned int i = 2; i < 4; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->blockId()),false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some of the leaves in the fifth child of the root
  child = LoadInnerNode(root->getChild(4)->blockId());
  for(unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), false, 4 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode() + 5, 4 * nodeStore->layout().maxChildrenPerInnerNode() + 5);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddlePartOfThreelevelTree_AlsoLastNonfullChild) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->getChild(1)->blockId());
  for(unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), false, nodeStore->layout().maxChildrenPerInnerNode() + i);
  }
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(unsigned int i = 2; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some of the leaves in the sixth child of the root
  child = LoadInnerNode(root->getChild(5)->blockId());
  for(unsigned int i = 0; i < 2; ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), false, 5 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode() + 5, 5 * nodeStore->layout().maxChildrenPerInnerNode() + 2);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseLastPartOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->getChild(1)->blockId());
  for(unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), false, nodeStore->layout().maxChildrenPerInnerNode() + i);
  }
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(unsigned int i = 2; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all of the leaves in the sixth child of the root
  child = LoadInnerNode(root->getChild(5)->blockId());
  for(unsigned int i = 0; i < child->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), i == child->numChildren()-1, 5 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode() + 5, 5 * nodeStore->layout().maxChildrenPerInnerNode() + child->numChildren());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->getChild(i)->blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all of the leaves in the sixth child of the root
  auto child = LoadInnerNode(root->getChild(5)->blockId());
  for(unsigned int i = 0; i < child->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->getChild(i)->blockId(), i==child->numChildren()-1, 5 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(root.get(), 0, 5 * nodeStore->layout().maxChildrenPerInnerNode() + child->numChildren());
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseAllLeavesOfFourLevelTree) {
  auto root = CreateFourLevel();
  //Traverse all leaves of the full threelevel tree in the first child
  auto firstChild = LoadInnerNode(root->getChild(0)->blockId());
  for(unsigned int i = 0; i < firstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(firstChild->getChild(i)->blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all leaves of the full threelevel tree in the second child
  auto secondChild = LoadInnerNode(root->getChild(1)->blockId());
  for(unsigned int i = 0; i < secondChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(secondChild->getChild(i)->blockId()), false, (nodeStore->layout().maxChildrenPerInnerNode() + i) * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all leaves of the non-full threelevel tree in the third child
  auto thirdChild = LoadInnerNode(root->getChild(2)->blockId());
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(thirdChild->getChild(0)->blockId()), false, 2 * nodeStore->layout().maxChildrenPerInnerNode() * nodeStore->layout().maxChildrenPerInnerNode());
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(thirdChild->getChild(1)->blockId())->getChild(0)->blockId(), true, 2 * nodeStore->layout().maxChildrenPerInnerNode() * nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(root.get(), 0, 2*nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxChildrenPerInnerNode() + 1);
}

TEST_F(DataTreeTest_TraverseLeaves, TraverseMiddlePartOfFourLevelTree) {
  auto root = CreateFourLevel();
  //Traverse some leaves of the full threelevel tree in the first child
  auto firstChild = LoadInnerNode(root->getChild(0)->blockId());
  auto secondChildOfFirstChild = LoadInnerNode(firstChild->getChild(1)->blockId());
  for(unsigned int i = 5; i < secondChildOfFirstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(secondChildOfFirstChild->getChild(i)->blockId(), false, nodeStore->layout().maxChildrenPerInnerNode()+i);
  }
  for(unsigned int i = 2; i < firstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(firstChild->getChild(i)->blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all leaves of the full threelevel tree in the second child
  auto secondChild = LoadInnerNode(root->getChild(1)->blockId());
  for(unsigned int i = 0; i < secondChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(secondChild->getChild(i)->blockId()), false, (nodeStore->layout().maxChildrenPerInnerNode() + i) * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some leaves of the non-full threelevel tree in the third child
  auto thirdChild = LoadInnerNode(root->getChild(2)->blockId());
  auto firstChildOfThirdChild = LoadInnerNode(thirdChild->getChild(0)->blockId());
  for(unsigned int i = 0; i < firstChildOfThirdChild->numChildren()-1; ++i) {
    EXPECT_TRAVERSE_LEAF(firstChildOfThirdChild->getChild(i)->blockId(), false, 2 * nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode()+i);
  }

  TraverseLeaves(root.get(), nodeStore->layout().maxChildrenPerInnerNode()+5, 2*nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxChildrenPerInnerNode() -1);
}

TEST_F(DataTreeTest_TraverseLeaves, LastLeafIsAlreadyResizedInCallback) {
  auto root = CreateLeaf();
  root->flush();
  auto tree = treeStore.load(root->blockId()).value();
  tree->traverseLeaves(0, 2, [this] (uint32_t leafIndex, bool /*isRightBorderNode*/, LeafHandle leaf) {
      if (leafIndex == 0) {
        EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf.node()->numBytes());
      } else {
        EXPECT_TRUE(false) << "only two nodes";
      }
  }, [] (uint32_t /*nodeIndex*/) -> Data {
      return Data(1);
  });
}

TEST_F(DataTreeTest_TraverseLeaves, LastLeafIsAlreadyResizedInCallback_TwoLevel) {
  auto root = CreateFullTwoLevelWithLastLeafSize(5);
  root->flush();
  auto tree = treeStore.load(root->blockId()).value();
  tree->traverseLeaves(0, nodeStore->layout().maxChildrenPerInnerNode()+1, [this] (uint32_t /*leafIndex*/, bool /*isRightBorderNode*/, LeafHandle leaf) {
      EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf.node()->numBytes());
  }, [] (uint32_t /*nodeIndex*/) -> Data {
      return Data(1);
  });
}

TEST_F(DataTreeTest_TraverseLeaves, ResizeFromOneLeafToMultipleLeaves) {
  auto root = CreateLeaf();
  EXPECT_TRAVERSE_LEAF(root->blockId(), false, 0);
  //EXPECT_CALL(traversor, calledExistingLeaf(_, false, 0)).Times(1);
  for (uint32_t i = 1; i < 10; ++i) {
    EXPECT_CREATE_LEAF(i);
  }
  TraverseLeaves(root.get(), 0, 10);
}

//TODO Refactor the test cases that are too long
