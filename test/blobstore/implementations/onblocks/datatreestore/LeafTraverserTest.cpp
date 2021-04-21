#include "testutils/DataTreeTest.h"
#include <blobstore/implementations/onblocks/datatreestore/impl/LeafTraverser.h>
#include <gmock/gmock.h>

using ::testing::Invoke;
using ::testing::Eq;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datatreestore::LeafHandle;
using blobstore::onblocks::datatreestore::LeafTraverser;
using blockstore::BlockId;

using cpputils::unique_ref;
using cpputils::Data;
using std::shared_ptr;
using std::make_shared;

class TraversorMock {
public:
  MOCK_METHOD(void, calledExistingLeaf, (DataLeafNode*, bool, uint32_t));
  MOCK_METHOD(shared_ptr<Data>, calledCreateLeaf, (uint32_t));
};

MATCHER_P(KeyEq, expected, "node blockId equals") {
  return arg->blockId() == expected;
}

class LeafTraverserTest: public DataTreeTest {
public:
  LeafTraverserTest() :traversor() {}

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
      EXPECT_TRAVERSE_LEAF(node.readChild(i).blockId(), isRightBorderNode && i == node.numChildren()-1, firstLeafIndex+i);
    }
  }

  void EXPECT_DONT_TRAVERSE_ANY_LEAVES() {
    EXPECT_CALL(traversor, calledExistingLeaf(testing::_, testing::_, testing::_)).Times(0);
    EXPECT_CALL(traversor, calledCreateLeaf(testing::_)).Times(0);
  }

  void TraverseLeaves(unique_ref<DataNode> root, uint32_t beginIndex, uint32_t endIndex, bool expectReadOnly) {
    root->flush();
    auto tree = treeStore.load(root->blockId()).value();
    auto* old_root = root.get();
    LeafTraverser(nodeStore, expectReadOnly).traverseAndUpdateRoot(&root, beginIndex, endIndex, [this] (uint32_t nodeIndex, bool isRightBorderNode,LeafHandle leaf) {
      traversor.calledExistingLeaf(leaf.node(), isRightBorderNode,  nodeIndex);
    }, [this] (uint32_t nodeIndex) -> Data {
        return traversor.calledCreateLeaf(nodeIndex)->copy();
    }, [] (auto) {});
    if (expectReadOnly) {
      EXPECT_EQ(old_root, root.get());
    } else {
      EXPECT_NE(old_root, root.get());
    }
  }

  TraversorMock traversor;
};

TEST_F(LeafTraverserTest, TraverseSingleLeafTree) {
  unique_ref<DataNode> root = CreateLeaf();
  EXPECT_TRAVERSE_LEAF(root->blockId(), true, 0);

  TraverseLeaves(std::move(root), 0, 1, true);
}

TEST_F(LeafTraverserTest, TraverseNothingInSingleLeafTree1) {
  unique_ref<DataNode> root = CreateLeaf();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(std::move(root), 0, 0, true);
}

TEST_F(LeafTraverserTest, TraverseNothingInSingleLeafTree2) {
  unique_ref<DataNode> root = CreateLeaf();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(std::move(root), 1, 1, true);
}

TEST_F(LeafTraverserTest, TraverseFirstLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->readChild(0).blockId(), false, 0);

  TraverseLeaves(std::move(root), 0, 1, true);
}

TEST_F(LeafTraverserTest, TraverseMiddleLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->readChild(5).blockId(), false, 5);

  TraverseLeaves(std::move(root), 5, 6, true);
}

TEST_F(LeafTraverserTest, TraverseLastLeafOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_LEAF(root->readChild(nodeStore->layout().maxChildrenPerInnerNode()-1).blockId(), true, nodeStore->layout().maxChildrenPerInnerNode()-1);

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode()-1, nodeStore->layout().maxChildrenPerInnerNode(), true);
}

TEST_F(LeafTraverserTest, TraverseNothingInFullTwolevelTree1) {
  auto root = CreateFullTwoLevel();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(std::move(root), 0, 0, true);
}

TEST_F(LeafTraverserTest, TraverseNothingInFullTwolevelTree2) {
  auto root = CreateFullTwoLevel();
  EXPECT_DONT_TRAVERSE_ANY_LEAVES();

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode(), nodeStore->layout().maxChildrenPerInnerNode(), true);
}

TEST_F(LeafTraverserTest, TraverseFirstLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(0).blockId())->readChild(0).blockId(), false, 0);

  TraverseLeaves(std::move(root), 0, 1, true);
}

TEST_F(LeafTraverserTest, TraverseMiddleLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(0).blockId())->readChild(5).blockId(), false, 5);

  TraverseLeaves(std::move(root), 5, 6, true);
}

TEST_F(LeafTraverserTest, TraverseLastLeafOfThreeLevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(1).blockId())->readChild(0).blockId(), true, nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode(), nodeStore->layout().maxChildrenPerInnerNode()+1, true);
}

TEST_F(LeafTraverserTest, TraverseAllLeavesOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*root, true, 0);

  TraverseLeaves(std::move(root), 0, nodeStore->layout().maxChildrenPerInnerNode(), true);
}

TEST_F(LeafTraverserTest, TraverseAllLeavesOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(0).blockId()), false, 0);
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(1).blockId())->readChild(0).blockId(), true, nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(std::move(root), 0, nodeStore->layout().maxChildrenPerInnerNode()+1, true);
}

TEST_F(LeafTraverserTest, TraverseFirstChildOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(0).blockId()), false, 0);

  TraverseLeaves(std::move(root), 0, nodeStore->layout().maxChildrenPerInnerNode(), true);
}

TEST_F(LeafTraverserTest, TraverseFirstPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(root->readChild(i).blockId(), false, i);
  }

  TraverseLeaves(std::move(root), 0, 5, true);
}

TEST_F(LeafTraverserTest, TraverseInnerPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (unsigned int i = 5; i < 10; ++i) {
    EXPECT_TRAVERSE_LEAF(root->readChild(i).blockId(), false, i);
  }

  TraverseLeaves(std::move(root), 5, 10, true);
}

TEST_F(LeafTraverserTest, TraverseLastPartOfFullTwolevelTree) {
  auto root = CreateFullTwoLevel();
  for (unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(root->readChild(i).blockId(), i==nodeStore->layout().maxChildrenPerInnerNode()-1, i);
  }

  TraverseLeaves(std::move(root), 5, nodeStore->layout().maxChildrenPerInnerNode(), true);
}

TEST_F(LeafTraverserTest, TraverseFirstPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->readChild(0).blockId());
  for (unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(node->readChild(i).blockId(), false, i);
  }

  TraverseLeaves(std::move(root), 0, 5, true);
}

TEST_F(LeafTraverserTest, TraverseInnerPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->readChild(0).blockId());
  for (unsigned int i = 5; i < 10; ++i) {
    EXPECT_TRAVERSE_LEAF(node->readChild(i).blockId(), false, i);
  }

  TraverseLeaves(std::move(root), 5, 10, true);
}

TEST_F(LeafTraverserTest, TraverseLastPartOfThreelevelMinDataTree) {
  auto root = CreateThreeLevelMinData();
  auto node = LoadInnerNode(root->readChild(0).blockId());
  for (unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(node->readChild(i).blockId(), false, i);
  }
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(1).blockId())->readChild(0).blockId(), true, nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(std::move(root), 5, nodeStore->layout().maxChildrenPerInnerNode()+1, true);
}

TEST_F(LeafTraverserTest, TraverseFirstLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(0).blockId())->readChild(0).blockId(), false, 0);

  TraverseLeaves(std::move(root), 0, 1, true);
}

TEST_F(LeafTraverserTest, TraverseLastLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  uint32_t numLeaves = nodeStore->layout().maxChildrenPerInnerNode() * 5 + 3;
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readLastChild().blockId())->readLastChild().blockId(), true, numLeaves-1);

  TraverseLeaves(std::move(root), numLeaves-1, numLeaves, true);
}

TEST_F(LeafTraverserTest, TraverseMiddleLeafOfThreelevelTree) {
  auto root = CreateThreeLevel();
  uint32_t wantedLeafIndex = nodeStore->layout().maxChildrenPerInnerNode() * 2 + 5;
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(root->readChild(2).blockId())->readChild(5).blockId(), false, wantedLeafIndex);

  TraverseLeaves(std::move(root), wantedLeafIndex, wantedLeafIndex+1, true);
}

TEST_F(LeafTraverserTest, TraverseFirstPartOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse all leaves in the first two children of the root
  for(unsigned int i = 0; i < 2; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(i).blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some of the leaves in the third child of the root
  auto child = LoadInnerNode(root->readChild(2).blockId());
  for(unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), false, 2 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(std::move(root), 0, 2 * nodeStore->layout().maxChildrenPerInnerNode() + 5, true);
}

TEST_F(LeafTraverserTest, TraverseMiddlePartOfThreelevelTree_OnlyFullChildren) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->readChild(1).blockId());
  for(unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), false, nodeStore->layout().maxChildrenPerInnerNode() + i);
  }
  //Traverse all leaves in the third and fourth child of the root
  for(unsigned int i = 2; i < 4; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(i).blockId()),false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some of the leaves in the fifth child of the root
  child = LoadInnerNode(root->readChild(4).blockId());
  for(unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), false, 4 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode() + 5, 4 * nodeStore->layout().maxChildrenPerInnerNode() + 5, true);
}

TEST_F(LeafTraverserTest, TraverseMiddlePartOfThreelevelTree_AlsoLastNonfullChild) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->readChild(1).blockId());
  for(unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), false, nodeStore->layout().maxChildrenPerInnerNode() + i);
  }
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(unsigned int i = 2; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(i).blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some of the leaves in the sixth child of the root
  child = LoadInnerNode(root->readChild(5).blockId());
  for(unsigned int i = 0; i < 2; ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), false, 5 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode() + 5, 5 * nodeStore->layout().maxChildrenPerInnerNode() + 2, true);
}

TEST_F(LeafTraverserTest, TraverseLastPartOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse some of the leaves in the second child of the root
  auto child = LoadInnerNode(root->readChild(1).blockId());
  for(unsigned int i = 5; i < nodeStore->layout().maxChildrenPerInnerNode(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), false, nodeStore->layout().maxChildrenPerInnerNode() + i);
  }
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(unsigned int i = 2; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(i).blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all of the leaves in the sixth child of the root
  child = LoadInnerNode(root->readChild(5).blockId());
  for(unsigned int i = 0; i < child->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), i == child->numChildren()-1, 5 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode() + 5, 5 * nodeStore->layout().maxChildrenPerInnerNode() + child->numChildren(), true);
}

TEST_F(LeafTraverserTest, TraverseAllLeavesOfThreelevelTree) {
  auto root = CreateThreeLevel();
  //Traverse all leaves in the third, fourth and fifth child of the root
  for(unsigned int i = 0; i < 5; ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(root->readChild(i).blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all of the leaves in the sixth child of the root
  auto child = LoadInnerNode(root->readChild(5).blockId());
  for(unsigned int i = 0; i < child->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(child->readChild(i).blockId(), i==child->numChildren()-1, 5 * nodeStore->layout().maxChildrenPerInnerNode() + i);
  }

  TraverseLeaves(std::move(root), 0, 5 * nodeStore->layout().maxChildrenPerInnerNode() + child->numChildren(), true);
}

TEST_F(LeafTraverserTest, TraverseAllLeavesOfFourLevelTree) {
  auto root = CreateFourLevel();
  //Traverse all leaves of the full threelevel tree in the first child
  auto firstChild = LoadInnerNode(root->readChild(0).blockId());
  for(unsigned int i = 0; i < firstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(firstChild->readChild(i).blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all leaves of the full threelevel tree in the second child
  auto secondChild = LoadInnerNode(root->readChild(1).blockId());
  for(unsigned int i = 0; i < secondChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(secondChild->readChild(i).blockId()), false, (nodeStore->layout().maxChildrenPerInnerNode() + i) * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all leaves of the non-full threelevel tree in the third child
  auto thirdChild = LoadInnerNode(root->readChild(2).blockId());
  EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(thirdChild->readChild(0).blockId()), false, 2 * nodeStore->layout().maxChildrenPerInnerNode() * nodeStore->layout().maxChildrenPerInnerNode());
  EXPECT_TRAVERSE_LEAF(LoadInnerNode(thirdChild->readChild(1).blockId())->readChild(0).blockId(), true, 2 * nodeStore->layout().maxChildrenPerInnerNode() * nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxChildrenPerInnerNode());

  TraverseLeaves(std::move(root), 0, 2*nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxChildrenPerInnerNode() + 1, true);
}

TEST_F(LeafTraverserTest, TraverseMiddlePartOfFourLevelTree) {
  auto root = CreateFourLevel();
  //Traverse some leaves of the full threelevel tree in the first child
  auto firstChild = LoadInnerNode(root->readChild(0).blockId());
  auto secondChildOfFirstChild = LoadInnerNode(firstChild->readChild(1).blockId());
  for(unsigned int i = 5; i < secondChildOfFirstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_LEAF(secondChildOfFirstChild->readChild(i).blockId(), false, nodeStore->layout().maxChildrenPerInnerNode()+i);
  }
  for(unsigned int i = 2; i < firstChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(firstChild->readChild(i).blockId()), false, i * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse all leaves of the full threelevel tree in the second child
  auto secondChild = LoadInnerNode(root->readChild(1).blockId());
  for(unsigned int i = 0; i < secondChild->numChildren(); ++i) {
    EXPECT_TRAVERSE_ALL_CHILDREN_OF(*LoadInnerNode(secondChild->readChild(i).blockId()), false, (nodeStore->layout().maxChildrenPerInnerNode() + i) * nodeStore->layout().maxChildrenPerInnerNode());
  }
  //Traverse some leaves of the non-full threelevel tree in the third child
  auto thirdChild = LoadInnerNode(root->readChild(2).blockId());
  auto firstChildOfThirdChild = LoadInnerNode(thirdChild->readChild(0).blockId());
  for(unsigned int i = 0; i < firstChildOfThirdChild->numChildren()-1; ++i) {
    EXPECT_TRAVERSE_LEAF(firstChildOfThirdChild->readChild(i).blockId(), false, 2 * nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode()+i);
  }

  TraverseLeaves(std::move(root), nodeStore->layout().maxChildrenPerInnerNode()+5, 2*nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxChildrenPerInnerNode() -1, true);
}

TEST_F(LeafTraverserTest, LastLeafIsAlreadyResizedInCallback) {
  unique_ref<DataNode> root = CreateLeaf();
  root->flush();
  auto* old_root = root.get();
  auto tree = treeStore.load(root->blockId()).value();
  LeafTraverser(nodeStore, false).traverseAndUpdateRoot(&root, 0, 2, [this] (uint32_t leafIndex, bool /*isRightBorderNode*/, LeafHandle leaf) {
      if (leafIndex == 0) {
        EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf.node()->numBytes());
      } else {
        EXPECT_TRUE(false) << "only two nodes";
      }
  }, [] (uint32_t /*nodeIndex*/) -> Data {
      return Data(1);
  }, [] (auto) {});
  EXPECT_NE(old_root, root.get()); // expect that we grew the tree
}

TEST_F(LeafTraverserTest, LastLeafIsAlreadyResizedInCallback_TwoLevel) {
  unique_ref<DataNode> root = CreateFullTwoLevelWithLastLeafSize(5);
  root->flush();
  auto* old_root = root.get();
  auto tree = treeStore.load(root->blockId()).value();
  LeafTraverser(nodeStore, false).traverseAndUpdateRoot(&root, 0, nodeStore->layout().maxChildrenPerInnerNode()+1, [this] (uint32_t /*leafIndex*/, bool /*isRightBorderNode*/, LeafHandle leaf) {
      EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf.node()->numBytes());
  }, [] (uint32_t /*nodeIndex*/) -> Data {
      return Data(1);
  }, [] (auto) {});
  EXPECT_NE(old_root, root.get()); // expect that we grew the tree
}

TEST_F(LeafTraverserTest, ResizeFromOneLeafToMultipleLeaves) {
  auto root = CreateLeaf();
  EXPECT_TRAVERSE_LEAF(root->blockId(), false, 0);
  //EXPECT_CALL(traversor, calledExistingLeaf(_, false, 0)).Times(1);
  for (uint32_t i = 1; i < 10; ++i) {
    EXPECT_CREATE_LEAF(i);
  }
  TraverseLeaves(std::move(root), 0, 10, false);
}

////TODO Refactor the test cases that are too long
