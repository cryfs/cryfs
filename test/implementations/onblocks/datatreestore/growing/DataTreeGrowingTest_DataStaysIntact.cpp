#include "testutils/DataTreeGrowingTest.h"
#include "../testutils/LeafDataFixture.h"
#include "../testutils/TwoLevelDataFixture.h"

using std::unique_ptr;
using std::make_unique;

using ::testing::WithParamInterface;
using ::testing::Values;

using namespace blobstore::onblocks::datanodestore;
using namespace blobstore::onblocks::datatreestore;

using blockstore::Key;
using cpputils::dynamic_pointer_move;

class DataTreeGrowingTest_DataStaysIntact: public DataTreeGrowingTest {
public:
  unique_ptr<DataTree> CreateLeafOnlyTreeWithData(TwoLevelDataFixture *data) {
    auto leafnode = nodeStore.createNewLeafNode();
    data->FillInto(leafnode.get());

    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  unique_ptr<DataTree> CreateTwoNodeTreeWithData(TwoLevelDataFixture *data) {
    auto root = CreateInner({CreateLeaf(), CreateLeaf()});
    data->FillInto(root.get());
    return make_unique<DataTree>(&nodeStore, std::move(root));
  }

  unique_ptr<DataTree> CreateThreeNodeChainedTreeWithData(TwoLevelDataFixture *data) {
    auto leaf = nodeStore.createNewLeafNode();
    data->FillInto(leaf.get());

    auto inner = nodeStore.createNewInnerNode(*leaf);
    return make_unique<DataTree>(&nodeStore, nodeStore.createNewInnerNode(*inner));
  }

  unique_ptr<DataTree> CreateFullTwoLevelTreeWithData(TwoLevelDataFixture *data) {
    auto root = CreateFullTwoLevel();
    assert(root->numChildren() == DataInnerNode::MAX_STORED_CHILDREN);
    data->FillInto(root.get());
    return make_unique<DataTree>(&nodeStore, std::move(root));
  }

  unique_ptr<DataTree> CreateThreeLevelTreeWithLowerLevelFullWithData(TwoLevelDataFixture *data) {
    auto root = CreateInner({CreateFullTwoLevel()});
    data->FillInto(root.get());
    return make_unique<DataTree>(&nodeStore, std::move(root));
  }
};

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAFullTwoLevelTree) {
  TwoLevelDataFixture data(&nodeStore);
  auto tree = CreateFullTwoLevelTreeWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto root = LoadInnerNode(tree->key());
  data.EXPECT_DATA_CORRECT(dynamic_pointer_move<DataInnerNode>(root).get(), DataInnerNode::MAX_STORED_CHILDREN);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeLevelTreeWithLowerLevelFull) {
  TwoLevelDataFixture data(&nodeStore);
  auto tree = CreateThreeLevelTreeWithLowerLevelFullWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto root = LoadInnerNode(tree->key());
  data.EXPECT_DATA_CORRECT(dynamic_pointer_move<DataInnerNode>(root).get(), DataInnerNode::MAX_STORED_CHILDREN);
}

class DataTreeGrowingTest_DataStaysIntact_OneDataLeaf: public DataTreeGrowingTest_DataStaysIntact, public WithParamInterface<uint32_t> {
};
INSTANTIATE_TEST_CASE_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, Values(0, 1, DataLeafNode::MAX_STORED_BYTES-2, DataLeafNode::MAX_STORED_BYTES-1, DataLeafNode::MAX_STORED_BYTES));

TEST_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, GrowAOneNodeTree) {
  TwoLevelDataFixture data(&nodeStore, GetParam(), true);
  auto tree = CreateLeafOnlyTreeWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto root = LoadInnerNode(tree->key());
  data.EXPECT_DATA_CORRECT(root.get(), 1);
}

TEST_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, GrowATwoNodeTree) {
  TwoLevelDataFixture data(&nodeStore, GetParam(), true);
  auto tree = CreateTwoNodeTreeWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto root = LoadInnerNode(tree->key());
  data.EXPECT_DATA_CORRECT(root.get(), 2);
}

TEST_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, GrowAThreeNodeChainedTree) {
  TwoLevelDataFixture data(&nodeStore, GetParam(), true);
  auto tree = CreateThreeNodeChainedTreeWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto root = LoadInnerNode(tree->key());
  data.EXPECT_DATA_CORRECT(root.get(), 1);
}
