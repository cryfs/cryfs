#include "testutils/DataTreeTest.h"
#include "testutils/TwoLevelDataFixture.h"
#include "blobstore/implementations/onblocks/utils/Math.h"
#include <cpp-utils/data/Data.h>

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
using blockstore::BlockId;
using cpputils::Data;
using boost::none;

using cpputils::unique_ref;

class DataTreeTest_ResizeNumBytes: public DataTreeTest {
public:
  static constexpr DataNodeLayout LAYOUT = DataNodeLayout(BLOCKSIZE_BYTES);

  unique_ref<DataTree> CreateTree(unique_ref<DataNode> root) {
    BlockId blockId = root->blockId();
    cpputils::destruct(std::move(root));
    return treeStore.load(blockId).value();
  }

  unique_ref<DataTree> CreateLeafTreeWithSize(uint32_t size) {
    return CreateTree(CreateLeafWithSize(size));
  }

  unique_ref<DataTree> CreateTwoLeafTreeWithSecondLeafSize(uint32_t size) {
    return CreateTree(CreateTwoLeafWithSecondLeafSize(size));
  }

  unique_ref<DataTree> CreateFullTwoLevelTreeWithLastLeafSize(uint32_t size) {
    return CreateTree(CreateFullTwoLevelWithLastLeafSize(size));
  }

  unique_ref<DataTree> CreateThreeLevelTreeWithTwoChildrenAndLastLeafSize(uint32_t size) {
    return CreateTree(CreateThreeLevelWithTwoChildrenAndLastLeafSize(size));
  }

  unique_ref<DataTree> CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(uint32_t size) {
    return CreateTree(CreateThreeLevelWithThreeChildrenAndLastLeafSize(size));
  }

  unique_ref<DataTree> CreateFullThreeLevelTreeWithLastLeafSize(uint32_t size) {
    return CreateTree(CreateFullThreeLevelWithLastLeafSize(size));
  }

  unique_ref<DataTree> CreateFourLevelMinDataTreeWithLastLeafSize(uint32_t size) {
    return CreateTree(CreateFourLevelMinDataWithLastLeafSize(size));
  }

  // NOLINTNEXTLINE(misc-no-recursion)
  void EXPECT_IS_LEFTMAXDATA_TREE(const BlockId &blockId) {
    auto root = nodeStore->load(blockId).value();
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root.get());
    if (inner != nullptr) {
      for (uint32_t i = 0; i < inner->numChildren()-1; ++i) {
        EXPECT_IS_MAXDATA_TREE(inner->readChild(i).blockId());
      }
      EXPECT_IS_LEFTMAXDATA_TREE(inner->readLastChild().blockId());
    }
  }

  // NOLINTNEXTLINE(misc-no-recursion)
  void EXPECT_IS_MAXDATA_TREE(const BlockId &blockId) {
    auto root = nodeStore->load(blockId).value();
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root.get());
    if (inner != nullptr) {
      for (uint32_t i = 0; i < inner->numChildren(); ++i) {
        EXPECT_IS_MAXDATA_TREE(inner->readChild(i).blockId());
      }
    } else {
      DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root.get());
      EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf->numBytes());
    }
  }
};
constexpr DataNodeLayout DataTreeTest_ResizeNumBytes::LAYOUT;

class DataTreeTest_ResizeNumBytes_P: public DataTreeTest_ResizeNumBytes, public WithParamInterface<tuple<function<unique_ref<DataTree>(DataTreeTest_ResizeNumBytes*, uint32_t)>, uint32_t, uint32_t, uint32_t>> {
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

  void ResizeTree(const BlockId &blockId, uint64_t size) {
    treeStore.load(blockId).get()->resizeNumBytes(size);
  }

  // NOLINTNEXTLINE(misc-no-recursion)
  unique_ref<DataLeafNode> LastLeaf(const BlockId &blockId) {
    auto root = nodeStore->load(blockId).value();
    auto leaf = dynamic_pointer_move<DataLeafNode>(root);
    if (leaf != none) {
      return std::move(*leaf);
    }
    auto inner = dynamic_pointer_move<DataInnerNode>(root).value();
    return LastLeaf(inner->readLastChild().blockId());
  }

  uint32_t oldLastLeafSize;
  unique_ref<DataTree> tree;
  uint32_t newNumberOfLeaves;
  uint32_t newLastLeafSize;
  uint64_t newSize;
  Data ZEROES;
};
INSTANTIATE_TEST_SUITE_P(DataTreeTest_ResizeNumBytes_P, DataTreeTest_ResizeNumBytes_P,
  Combine(
    //Tree we're starting with
    Values<function<unique_ref<DataTree>(DataTreeTest_ResizeNumBytes*, uint32_t)>>(
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
  EXPECT_IS_LEFTMAXDATA_TREE(tree->blockId());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, NumBytesIsCorrect) {
  tree->resizeNumBytes(newSize);
  tree->flush();
  // tree->numBytes() only goes down the right border nodes and expects the tree to be a left max data tree.
  // This is what the StructureIsValid test case is for.
  EXPECT_EQ(newSize, tree->numBytes());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, NumLeavesIsCorrect) {
  tree->resizeNumBytes(newSize);
  tree->flush();
  // tree->numLeaves() only goes down the right border nodes and expects the tree to be a left max data tree.
  // This is what the StructureIsValid test case is for.
  EXPECT_EQ(newNumberOfLeaves, tree->forceComputeNumLeaves());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, NumLeavesIsCorrect_FromCache) {
  tree->numLeaves(); // fill cache with old value
  tree->resizeNumBytes(newSize);
  tree->flush();
  // tree->numLeaves() only goes down the right border nodes and expects the tree to be a left max data tree.
  // This is what the StructureIsValid test case is for.
  EXPECT_EQ(newNumberOfLeaves, tree->numLeaves());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, DepthFlagsAreCorrect) {
  tree->resizeNumBytes(newSize);
  tree->flush();
  uint32_t depth = ceil(log(newNumberOfLeaves)/log(DataTreeTest_ResizeNumBytes::LAYOUT.maxChildrenPerInnerNode()) - 0.00000000001); // The subtraction takes care of double inaccuracies if newNumberOfLeaves == maxChildrenPerInnerNode
  CHECK_DEPTH(depth, tree->blockId());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, KeyDoesntChange) {
  BlockId blockId = tree->blockId();
  tree->flush();
  tree->resizeNumBytes(newSize);
  EXPECT_EQ(blockId, tree->blockId());
}

TEST_P(DataTreeTest_ResizeNumBytes_P, DataStaysIntact) {
  uint32_t oldNumberOfLeaves = std::max(UINT64_C(1), ceilDivision(tree->numBytes(), static_cast<uint64_t>(nodeStore->layout().maxBytesPerLeaf())));
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Unchanged);
  BlockId blockId = tree->blockId();
  cpputils::destruct(std::move(tree));
  data.FillInto(nodeStore->load(blockId).get().get());

  ResizeTree(blockId, newSize);

  if (oldNumberOfLeaves < newNumberOfLeaves || (oldNumberOfLeaves == newNumberOfLeaves && oldLastLeafSize < newLastLeafSize)) {
    data.EXPECT_DATA_CORRECT(nodeStore->load(blockId).get().get(), oldNumberOfLeaves, oldLastLeafSize);
  } else {
    data.EXPECT_DATA_CORRECT(nodeStore->load(blockId).get().get(), newNumberOfLeaves, newLastLeafSize);
  }
}

TEST_P(DataTreeTest_ResizeNumBytes_P, UnneededBlocksGetDeletedWhenShrinking) {
    tree->resizeNumBytes(newSize);
    tree->flush();

    uint64_t expectedNumNodes = 1; // 1 for the root node
    uint64_t nodesOnCurrentLevel = newNumberOfLeaves;
    while (nodesOnCurrentLevel > 1) {
      expectedNumNodes += nodesOnCurrentLevel;
      nodesOnCurrentLevel = ceilDivision(nodesOnCurrentLevel, nodeStore->layout().maxChildrenPerInnerNode());
    }
    EXPECT_EQ(expectedNumNodes, nodeStore->numNodes());
}

//Resize to zero is not caught in the parametrized test above, in the following, we test it separately.

TEST_F(DataTreeTest_ResizeNumBytes, ResizeToZero_NumBytesIsCorrect) {
  auto tree = CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(10u);
  tree->resizeNumBytes(0);
  BlockId blockId = tree->blockId();
  cpputils::destruct(std::move(tree));
  auto leaf = LoadLeafNode(blockId);
  EXPECT_EQ(0u, leaf->numBytes());
}

TEST_F(DataTreeTest_ResizeNumBytes, ResizeToZero_blockIdDoesntChange) {
  auto tree = CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(10u);
  BlockId blockId = tree->blockId();
  tree->resizeNumBytes(0);
  tree->flush();
  EXPECT_EQ(blockId, tree->blockId());
}

TEST_F(DataTreeTest_ResizeNumBytes, ResizeToZero_UnneededBlocksGetDeletedWhenShrinking) {
  auto tree = CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize(10u);
  tree->resizeNumBytes(0);
  tree->flush();
  EXPECT_EQ(1u, nodeStore->numNodes());
}
