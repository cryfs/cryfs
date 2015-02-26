#include "../testutils/DataTreeTest.h"
#include <google/gmock/gmock.h>

using ::testing::_;
using ::testing::WithParamInterface;
using ::testing::Values;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blobstore::onblocks::datatreestore::DataTree;
using blockstore::Key;

using std::unique_ptr;

class DataTreeTest_NumStoredBytes: public DataTreeTest {
public:
  unique_ptr<DataLeafNode> CreateLeafWithSize(uint32_t size) {
    auto leaf = CreateLeaf();
    leaf->resize(size);
    return leaf;
  }

  unique_ptr<DataInnerNode> CreateTwoLeafWithSecondLeafSize(uint32_t size) {
    return CreateInner({
      CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateLeafWithSize(size)
    });
  }

  unique_ptr<DataInnerNode> CreateFullTwoLevelWithLastLeafSize(uint32_t size) {
    auto root = CreateFullTwoLevel();
    for (int i = 0; i < root->numChildren()-1; ++i) {
      LoadLeafNode(root->getChild(i)->key())->resize(nodeStore->layout().maxBytesPerLeaf());
    }
    LoadLeafNode(root->LastChild()->key())->resize(size);
    return root;
  }

  unique_ptr<DataInnerNode> CreateThreeLevelWithOneChildAndLastLeafSize(uint32_t size) {
    return CreateInner({
      CreateInner({
        CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
        CreateLeafWithSize(size)
      })
    });
  }

  unique_ptr<DataInnerNode> CreateThreeLevelWithTwoChildrenAndLastLeafSize(uint32_t size) {
    return CreateInner({
      CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateInner({
        CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
        CreateLeafWithSize(size)
      })
    });
  }

  unique_ptr<DataInnerNode> CreateThreeLevelWithThreeChildrenAndLastLeafSize(uint32_t size) {
    return CreateInner({
      CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateFullTwoLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateInner({
        CreateLeafWithSize(nodeStore->layout().maxBytesPerLeaf()),
        CreateLeafWithSize(size)
      })
    });
  }

  unique_ptr<DataInnerNode> CreateFullThreeLevelWithLastLeafSize(uint32_t size) {
    auto root = CreateFullThreeLevel();
    for (int i = 0; i < root->numChildren(); ++i) {
      auto node = LoadInnerNode(root->getChild(i)->key());
      for (int j = 0; j < node->numChildren(); ++j) {
        LoadLeafNode(node->getChild(j)->key())->resize(nodeStore->layout().maxBytesPerLeaf());
      }
    }
    LoadLeafNode(LoadInnerNode(root->LastChild()->key())->LastChild()->key())->resize(size);
    return root;
  }

  unique_ptr<DataInnerNode> CreateFourLevelMinDataWithLastLeafSize(uint32_t size) {
    return CreateInner({
      CreateFullThreeLevelWithLastLeafSize(nodeStore->layout().maxBytesPerLeaf()),
      CreateInner({CreateInner({CreateLeafWithSize(size)})})
    });
  }
};

TEST_F(DataTreeTest_NumStoredBytes, CreatedTreeIsEmpty) {
  auto tree = treeStore.createNewTree();
  EXPECT_EQ(0, tree->numStoredBytes());
}

class DataTreeTest_NumStoredBytes_P: public DataTreeTest_NumStoredBytes, public WithParamInterface<uint32_t> {};
INSTANTIATE_TEST_CASE_P(EmptyLastLeaf, DataTreeTest_NumStoredBytes_P, Values(0u));
INSTANTIATE_TEST_CASE_P(HalfFullLastLeaf, DataTreeTest_NumStoredBytes_P, Values(5u, 10u));
INSTANTIATE_TEST_CASE_P(FullLastLeaf, DataTreeTest_NumStoredBytes_P, Values(DataNodeLayout(DataTreeTest_NumStoredBytes::BLOCKSIZE_BYTES).maxBytesPerLeaf()));

TEST_P(DataTreeTest_NumStoredBytes_P, SingleLeaf) {
  Key key = CreateLeafWithSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, TwoLeafTree) {
  Key key = CreateTwoLeafWithSecondLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, FullTwolevelTree) {
  Key key = CreateFullTwoLevelWithLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*(nodeStore->layout().maxChildrenPerInnerNode()-1) + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, ThreeLevelTreeWithOneChild) {
  Key key = CreateThreeLevelWithOneChildAndLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, ThreeLevelTreeWithTwoChildren) {
  Key key = CreateThreeLevelWithTwoChildrenAndLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, ThreeLevelTreeWithThreeChildren) {
  Key key = CreateThreeLevelWithThreeChildrenAndLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(2*nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode() + nodeStore->layout().maxBytesPerLeaf() + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, FullThreeLevelTree) {
  Key key = CreateFullThreeLevelWithLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode()*(nodeStore->layout().maxChildrenPerInnerNode()-1) + nodeStore->layout().maxBytesPerLeaf()*(nodeStore->layout().maxChildrenPerInnerNode()-1) + GetParam(), tree->numStoredBytes());
}

TEST_P(DataTreeTest_NumStoredBytes_P, FourLevelMinDataTree) {
  Key key = CreateFourLevelMinDataWithLastLeafSize(GetParam())->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf()*nodeStore->layout().maxChildrenPerInnerNode()*nodeStore->layout().maxChildrenPerInnerNode() + GetParam(), tree->numStoredBytes());
}
