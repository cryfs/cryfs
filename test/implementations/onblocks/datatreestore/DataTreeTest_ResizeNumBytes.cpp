#include "testutils/DataTreeTest.h"
#include "testutils/TwoLevelDataFixture.h"
#include "../../../../implementations/onblocks/utils/Math.h"
#include <messmer/blockstore/utils/Data.h>

#include <tuple>

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Combine;
using std::tuple;
using std::get;
using std::function;
using std::mem_fn;
using cpputils::dynamic_pointer_move;

using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blobstore::onblocks::datatreestore::DataTree;
using blobstore::onblocks::utils::ceilDivision;
using blockstore::Key;
using blockstore::Data;

using std::unique_ptr;

class DataTreeTest_ResizeNumBytes: public DataTreeTest {
public:
  static constexpr DataNodeLayout LAYOUT = DataNodeLayout(BLOCKSIZE_BYTES);

  unique_ptr<DataTree> CreateTree(unique_ptr<DataNode> root) {
    Key key = root->key();
    root.reset();
    return treeStore.load(key);
  }

  unique_ptr<DataTree> CreateLeafTreeWithSize(uint32_t size) {
    return CreateTree(CreateLeafWithSize(size));
  }

  unique_ptr<DataTree> CreateTwoLeafTreeWithSecondLeafSize(uint32_t size) {
    return CreateTree(CreateTwoLeafWithSecondLeafSize(size));
  }

  unique_ptr<DataTree> CreateFullTwoLevelTreeWithLastLeafSize(uint32_t size) {
    return CreateTree(CreateFullTwoLevelWithLastLeafSize(size));
  }

  unique_ptr<DataTree> CreateThreeLevelTreeWithTwoChildrenAndLastLeafSize(uint32_t size) {
    return CreateTree(CreateThreeLevelWithTwoChildrenAndLastLeafSize(size));
  }

  unique_ptr<DataTree> CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(uint32_t size) {
    return CreateTree(CreateThreeLevelWithThreeChildrenAndLastLeafSize(size));
  }

  unique_ptr<DataTree> CreateFullThreeLevelTreeWithLastLeafSize(uint32_t size) {
    return CreateTree(CreateFullThreeLevelWithLastLeafSize(size));
  }

  unique_ptr<DataTree> CreateFourLevelMinDataTreeWithLastLeafSize(uint32_t size) {
    return CreateTree(CreateFourLevelMinDataWithLastLeafSize(size));
  }

  void EXPECT_IS_LEFTMAXDATA_TREE(const Key &key) {
    auto root = nodeStore->load(key);
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root.get());
    if (inner != nullptr) {
      for (int i = 0; i < inner->numChildren()-1; ++i) {
        EXPECT_IS_MAXDATA_TREE(inner->getChild(i)->key());
      }
      EXPECT_IS_LEFTMAXDATA_TREE(inner->LastChild()->key());
    }
  }

  void EXPECT_IS_MAXDATA_TREE(const Key &key) {
    auto root = nodeStore->load(key);
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root.get());
    if (inner != nullptr) {
      for (int i = 0; i < inner->numChildren(); ++i) {
        EXPECT_IS_MAXDATA_TREE(inner->getChild(i)->key());
      }
    } else {
      DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root.get());
      EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf->numBytes());
    }
  }
};
constexpr DataNodeLayout DataTreeTest_ResizeNumBytes::LAYOUT;

class DataTreeTest_ResizeNumBytes_P: public DataTreeTest_ResizeNumBytes, public WithParamInterface<tuple<function<unique_ptr<DataTree>(DataTreeTest_ResizeNumBytes*, uint32_t)>, uint32_t, uint32_t, uint32_t>> {
public:
  DataTreeTest_ResizeNumBytes_P()
    : oldLastLeafSize(get<1>(GetParam())),
      tree(get<0>(GetParam())(this, oldLastLeafSize)),
      newNumberOfLeaves(get<2>(GetParam())),
      newLastLeafSize(get<3>(GetParam())),
      newSize((newNumberOfLeaves-1) * LAYOUT.maxBytesPerLeaf() + newLastLeafSize),
      ZEROES(LAYOUT.maxBytesPerLeaf())
  {
    ZEROES.FillWithZeroes();
  }

  void ResizeTree(const Key &key, uint64_t size) {
    treeStore.load(key)->resizeNumBytes(size);
  }

  unique_ptr<DataLeafNode> LastLeaf(const Key &key) {
    auto root = nodeStore->load(key);
    auto leaf = dynamic_pointer_move<DataLeafNode>(root);
    if (leaf.get() != nullptr) {
      return leaf;
    }
    auto inner = dynamic_pointer_move<DataInnerNode>(root);
    return LastLeaf(inner->LastChild()->key());
  }

  uint32_t oldLastLeafSize;
  unique_ptr<DataTree> tree;
  uint32_t newNumberOfLeaves;
  uint32_t newLastLeafSize;
  uint64_t newSize;
  Data ZEROES;
};
INSTANTIATE_TEST_CASE_P(DataTreeTest_ResizeNumBytes_P, DataTreeTest_ResizeNumBytes_P,
  Combine(
    //Tree we're starting with
    Values<function<unique_ptr<DataTree>(DataTreeTest_ResizeNumBytes*, uint32_t)>>(
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateLeafTreeWithSize),
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateTwoLeafTreeWithSecondLeafSize),
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateFullTwoLevelTreeWithLastLeafSize),
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateThreeLevelTreeWithTwoChildrenAndLastLeafSize),
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize),
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateFullThreeLevelTreeWithLastLeafSize),
      mem_fn(&DataTreeTest_ResizeNumBytes::CreateFourLevelMinDataTreeWithLastLeafSize)
    ),
    //Last leaf size of the start tree
    Values(
      0u,
      1u,
      10u,
      DataTreeTest_ResizeNumBytes::LAYOUT.maxBytesPerLeaf()
    ),
    //Number of leaves we're resizing to
    Values(
      1u,
      2u,
      DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode(),  //Full two level tree
      2* DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode(), //Three level tree with two children
      3* DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode(), //Three level tree with three children
      DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode() * DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode(), //Full three level tree
      DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode() * DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode() + 1 //Four level mindata tree
    ),
    //Last leaf size of the resized tree
    Values(
      1u,
      10u,
      DataTreeTest_ResizeNumBytes::LAYOUT.maxBytesPerLeaf()
    )
  )
);

TEST_P(DataTreeTest_ResizeNumBytes_P, StructureIsValid) {
  tree->resizeNumBytes(newSize);
  tree->flush();
  EXPECT_IS_LEFTMAXDATA_TREE(tree->key());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, NumBytesIsCorrect) {
  tree->resizeNumBytes(newSize);
  tree->flush();
  // tree->numStoredBytes() only goes down the right border nodes and expects the tree to be a left max data tree.
  // This is what the StructureIsValid test case is for.
  EXPECT_EQ(newSize, tree->numStoredBytes());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, DepthFlagsAreCorrect) {
  tree->resizeNumBytes(newSize);
  tree->flush();
  uint32_t depth = ceil(log(newNumberOfLeaves)/log(DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode()));
  CHECK_DEPTH(depth, tree->key());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, KeyDoesntChange) {
  Key key = tree->key();
  tree->resizeNumBytes(newSize);
  tree->flush();
  EXPECT_EQ(key, tree->key());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, DataStaysIntact) {
  uint32_t oldNumberOfLeaves = std::max(1u, ceilDivision(tree->numStoredBytes(), nodeStore->layout().maxBytesPerLeaf()));
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Unchanged);
  Key key = tree->key();
  tree.reset();
  data.FillInto(nodeStore->load(key).get());

  ResizeTree(key, newSize);

  if (oldNumberOfLeaves < newNumberOfLeaves || (oldNumberOfLeaves == newNumberOfLeaves && oldLastLeafSize < newLastLeafSize)) {
    data.EXPECT_DATA_CORRECT(nodeStore->load(key).get(), oldNumberOfLeaves, oldLastLeafSize);
  } else {
    data.EXPECT_DATA_CORRECT(nodeStore->load(key).get(), newNumberOfLeaves, newLastLeafSize);
  }
}

TEST_P(DataTreeTest_ResizeNumBytes_P, UnusedEndOfLastLeafIsZero) {
  uint32_t oldNumberOfLeaves = std::max(1u, ceilDivision(tree->numStoredBytes(), nodeStore->layout().maxBytesPerLeaf()));
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Unchanged);
  Key key = tree->key();
  tree.reset();
  data.FillInto(nodeStore->load(key).get());

  ResizeTree(key, newSize);

  auto lastLeaf = LastLeaf(key);
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), (char*)lastLeaf->data()+lastLeaf->numBytes(), LAYOUT.maxBytesPerLeaf()-lastLeaf->numBytes()));
}


//Resize to zero is not caught in the parametrized test above, in the following, we test it separately.

TEST_F(DataTreeTest_ResizeNumBytes, ResizeToZero_NumBytesIsCorrect) {
  auto tree = CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(10u);
  tree->resizeNumBytes(0);
  Key key = tree->key();
  tree.reset();
  auto leaf = LoadLeafNode(key);
  EXPECT_EQ(0u, leaf->numBytes());
}

TEST_F(DataTreeTest_ResizeNumBytes, ResizeToZero_KeyDoesntChange) {
  auto tree = CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(10u);
  Key key = tree->key();
  tree->resizeNumBytes(0);
  EXPECT_EQ(key, tree->key());
}
