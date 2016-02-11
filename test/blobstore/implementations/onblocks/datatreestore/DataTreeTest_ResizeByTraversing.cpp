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
using blockstore::Key;
using cpputils::Data;
using boost::none;

using cpputils::unique_ref;

class DataTreeTest_ResizeByTraversing: public DataTreeTest {
public:
  static constexpr DataNodeLayout LAYOUT = DataNodeLayout(BLOCKSIZE_BYTES);

  unique_ref<DataTree> CreateTree(unique_ref<DataNode> root) {
    Key key = root->key();
    cpputils::destruct(std::move(root));
    return treeStore.load(key).value();
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

  void EXPECT_IS_LEFTMAXDATA_TREE(const Key &key) {
    auto root = nodeStore->load(key).value();
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root.get());
    if (inner != nullptr) {
      for (uint32_t i = 0; i < inner->numChildren()-1; ++i) {
        EXPECT_IS_MAXDATA_TREE(inner->getChild(i)->key());
      }
      EXPECT_IS_LEFTMAXDATA_TREE(inner->LastChild()->key());
    }
  }

  void EXPECT_IS_MAXDATA_TREE(const Key &key) {
    auto root = nodeStore->load(key).value();
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root.get());
    if (inner != nullptr) {
      for (uint32_t i = 0; i < inner->numChildren(); ++i) {
        EXPECT_IS_MAXDATA_TREE(inner->getChild(i)->key());
      }
    } else {
      DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root.get());
      EXPECT_EQ(nodeStore->layout().maxBytesPerLeaf(), leaf->numBytes());
    }
  }
};
constexpr DataNodeLayout DataTreeTest_ResizeByTraversing::LAYOUT;

class DataTreeTest_ResizeByTraversing_P: public DataTreeTest_ResizeByTraversing, public WithParamInterface<tuple<function<unique_ref<DataTree>(DataTreeTest_ResizeByTraversing*, uint32_t)>, uint32_t, uint32_t>> {
public:
  DataTreeTest_ResizeByTraversing_P()
    : oldLastLeafSize(get<1>(GetParam())),
      tree(get<0>(GetParam())(this, oldLastLeafSize)),
      numberOfLeavesToAdd(get<2>(GetParam())),
      newNumberOfLeaves(tree->numLeaves()+numberOfLeavesToAdd),
      ZEROES(LAYOUT.maxBytesPerLeaf())
  {
    ZEROES.FillWithZeroes();
  }

  void GrowTree(const Key &key, uint32_t numLeavesToAdd) {
    auto tree = treeStore.load(key);
    GrowTree(tree.get().get(), numLeavesToAdd);
  }

  void GrowTree(DataTree *tree, uint32_t numLeavesToAdd) {
    uint32_t oldNumLeaves = tree->numLeaves();
    uint32_t newNumLeaves = oldNumLeaves + numLeavesToAdd;
    //TODO Test cases where beginIndex is inside of the existing leaves
    tree->traverseLeaves(newNumLeaves-1, newNumLeaves, [] (DataLeafNode*,uint32_t){});
    tree->flush();
  }

  unique_ref<DataLeafNode> LastLeaf(const Key &key) {
    auto root = nodeStore->load(key).value();
    auto leaf = dynamic_pointer_move<DataLeafNode>(root);
    if (leaf != none) {
      return std::move(*leaf);
    }
    auto inner = dynamic_pointer_move<DataInnerNode>(root).value();
    return LastLeaf(inner->LastChild()->key());
  }

  uint32_t oldLastLeafSize;
  unique_ref<DataTree> tree;
  uint32_t numberOfLeavesToAdd;
  uint32_t newNumberOfLeaves;
  Data ZEROES;
};
INSTANTIATE_TEST_CASE_P(DataTreeTest_ResizeByTraversing_P, DataTreeTest_ResizeByTraversing_P,
  Combine(
    //Tree we're starting with
    Values<function<unique_ref<DataTree>(DataTreeTest_ResizeByTraversing*, uint32_t)>>(
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateLeafTreeWithSize),
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateTwoLeafTreeWithSecondLeafSize),
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateFullTwoLevelTreeWithLastLeafSize),
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateThreeLevelTreeWithTwoChildrenAndLastLeafSize),
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateThreeLevelTreeWithThreeChildrenAndLastLeafSize),
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateFullThreeLevelTreeWithLastLeafSize),
      mem_fn(&DataTreeTest_ResizeByTraversing::CreateFourLevelMinDataTreeWithLastLeafSize)
    ),
    //Last leaf size of the start tree
    Values(
      0u,
      1u,
      10u,
      DataTreeTest_ResizeByTraversing::LAYOUT.maxBytesPerLeaf()
    ),
    //Number of leaves we're adding
    Values(
      1u,
      2u,
      DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode(),  //Full two level tree
      2* DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode(), //Three level tree with two children
      3* DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode(), //Three level tree with three children
      DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode() * DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode(), //Full three level tree
      DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode() * DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode() + 1 //Four level mindata tree
    )
  )
);

TEST_P(DataTreeTest_ResizeByTraversing_P, StructureIsValid) {
  GrowTree(tree.get(), numberOfLeavesToAdd);
  EXPECT_IS_LEFTMAXDATA_TREE(tree->key());
}

TEST_P(DataTreeTest_ResizeByTraversing_P, NumBytesIsCorrect) {
  GrowTree(tree.get(), numberOfLeavesToAdd);
  // tree->numLeaves() only goes down the right border nodes and expects the tree to be a left max data tree.
  // This is what the StructureIsValid test case is for.
  EXPECT_EQ(newNumberOfLeaves, tree->numLeaves());
}

TEST_P(DataTreeTest_ResizeByTraversing_P, DepthFlagsAreCorrect) {
  GrowTree(tree.get(), numberOfLeavesToAdd);
  uint32_t depth = ceil(log(newNumberOfLeaves)/log(DataTreeTest_ResizeByTraversing::LAYOUT.maxChildrenPerInnerNode()));
  CHECK_DEPTH(depth, tree->key());
}

TEST_P(DataTreeTest_ResizeByTraversing_P, KeyDoesntChange) {
  Key key = tree->key();
  tree->flush();
  GrowTree(tree.get(), numberOfLeavesToAdd);
  EXPECT_EQ(key, tree->key());
}

TEST_P(DataTreeTest_ResizeByTraversing_P, DataStaysIntact) {
  uint32_t oldNumberOfLeaves = std::max(UINT64_C(1), ceilDivision(tree->numStoredBytes(), (uint64_t)nodeStore->layout().maxBytesPerLeaf()));
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Unchanged);
  Key key = tree->key();
  cpputils::destruct(std::move(tree));
  data.FillInto(nodeStore->load(key).get().get());

  GrowTree(key, newNumberOfLeaves);

  data.EXPECT_DATA_CORRECT(nodeStore->load(key).get().get(), oldNumberOfLeaves, oldLastLeafSize);
}
