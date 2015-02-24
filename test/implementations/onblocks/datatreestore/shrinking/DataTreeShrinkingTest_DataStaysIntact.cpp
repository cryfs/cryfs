#include "testutils/DataTreeShrinkingTest.h"
#include "../testutils/TwoLevelDataFixture.h"

using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

using std::make_unique;
using std::unique_ptr;
using std::function;

class DataTreeShrinkingTest_DataStaysIntact: public DataTreeShrinkingTest {
public:
  unique_ptr<DataTree> TreeWithData(unique_ptr<DataNode> root, TwoLevelDataFixture *data) {
    data->FillInto(root.get());
    Key key = root->key();
    root.reset();
    return treeStore.load(key);
  }

  void TestDataStaysIntactOnShrinking(unique_ptr<DataInnerNode> root, TwoLevelDataFixture *data) {
    auto tree = TreeWithData(std::move(root), data);
    tree->removeLastDataLeaf();
    tree->flush();

    data->EXPECT_DATA_CORRECT(nodeStore->load(tree->key()).get());
  }
};

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATwoLeafTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateTwoLeaf(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATwoLeafTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateTwoLeaf(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourNodeThreeLeafTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateFourNodeThreeLeaf(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourNodeThreeLeafTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateFourNodeThreeLeaf(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATwoInnerNodeOneTwoLeavesTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateTwoInnerNodeOneTwoLeaves(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATwoInnerNodeOneTwoLeavesTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateTwoInnerNodeOneTwoLeaves(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATwoInnerNodeTwoOneLeavesTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateTwoInnerNodeTwoOneLeaves(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATwoInnerNodeTwoOneLeavesTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateTwoInnerNodeTwoOneLeaves(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAThreeLevelMinDataTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateThreeLevelMinData(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAThreeLevelMinDataTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateThreeLevelMinData(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourLevelMinDataTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateFourLevelMinData(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourLevelMinDataTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateFourLevelMinData(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourLevelTreeWithTwoSiblingLeaves1_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateFourLevelWithTwoSiblingLeaves1(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourLevelTreeWithTwoSiblingLeaves1_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateFourLevelWithTwoSiblingLeaves1(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourLevelTreeWithTwoSiblingLeaves2_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateFourLevelWithTwoSiblingLeaves2(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAFourLevelTreeWithTwoSiblingLeaves2_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateFourLevelWithTwoSiblingLeaves2(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkATreeWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateWithFirstChildOfRootFullThreelevelAndSecondChildMindataThreelevel(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAThreeLevelTreeWithThreeChildrenOfRoot_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, true);
  TestDataStaysIntactOnShrinking(CreateThreeLevelWithThreeChildrenOfRoot(), &data);
}

TEST_F(DataTreeShrinkingTest_DataStaysIntact, ShrinkAThreeLevelTreeWithThreeChildrenOfRoot_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, 0, false);
  TestDataStaysIntactOnShrinking(CreateThreeLevelWithThreeChildrenOfRoot(), &data);
}
