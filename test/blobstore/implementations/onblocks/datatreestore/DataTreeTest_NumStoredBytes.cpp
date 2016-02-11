#include "testutils/DataTreeTest.h"
#include <gmock/gmock.h>

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

class DataTreeTest_NumStoredBytes: public DataTreeTest {
public:
};

TEST_F(DataTreeTest_NumStoredBytes, CreatedTreeIsEmpty) {
  auto tree = treeStore.createNewTree();
  EXPECT_EQ(0u, tree->numStoredBytes());
}

class DataTreeTest_NumStoredBytes_P: public DataTreeTest_NumStoredBytes, public WithParamInterface<uint32_t> {};
INSTANTIATE_TEST_CASE_P(EmptyLastLeaf, DataTreeTest_NumStoredBytes_P, Values(0u));
INSTANTIATE_TEST_CASE_P(HalfFullLastLeaf, DataTreeTest_NumStoredBytes_P, Values(5u, 10u));
INSTANTIATE_TEST_CASE_P(FullLastLeaf, DataTreeTest_NumStoredBytes_P, Values((uint32_t)DataNodeLayout(DataTreeTest_NumStoredBytes::BLOCKSIZE_BYTES).maxBytesPerLeaf()));

TEST_P(DataTreeTest_NumStoredBytes_P, SingleLeaf) {
  Key key = CreateLeafWithSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, TwoLeafTree) {
  Key key = CreateTwoLeafWithSecondLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, FullTwolevelTree) {
  Key key = CreateFullTwoLevelWithLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*(nodeStore->layout().maxChildrenPerInnerNode()-1) + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, ThreeLevelTreeWithOneChild) {
  Key key = CreateThreeLevelWithOneChildAndLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, ThreeLevelTreeWithTwoChildren) {
  Key key = CreateThreeLevelWithTwoChildrenAndLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, ThreeLevelTreeWithThreeChildren) {
  Key key = CreateThreeLevelWithThreeChildrenAndLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(2*nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, FullThreeLevelTree) {
  Key key = CreateFullThreeLevelWithLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode()*(nodeStore->layout().maxChildrenPerInnerNode()-1) + nodeStore->layout().maxBytesPerLeaf()*(nodeStore->layout().maxChildrenPerInnerNode()-1) + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, FourLevelMinDataTree) {
  Key key = CreateFourLevelMinDataWithLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode() + GetParam(), tree->numStoredBytes());
}
