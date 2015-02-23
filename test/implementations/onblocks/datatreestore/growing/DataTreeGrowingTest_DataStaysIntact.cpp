#include "testutils/DataTreeGrowingTest.h"
#include "testutils/LeafDataFixture.h"
#include "testutils/TwoLevelDataFixture.h"

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
  unique_ptr<DataTree> CreateLeafOnlyTreeWithData(const LeafDataFixture &data) {
    auto leafnode = nodeStore.createNewLeafNode();
    data.FillInto(leafnode.get());

    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  unique_ptr<DataTree> CreateTwoNodeTreeWithData(const LeafDataFixture &data) {
    auto tree = CreateLeafOnlyTreeWithData(data);
    tree->addDataLeaf();
    return tree;
  }

  unique_ptr<DataTree> CreateThreeNodeChainedTreeWithData(const LeafDataFixture &data) {
    auto leaf = nodeStore.createNewLeafNode();
    data.FillInto(leaf.get());

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
    auto _node = LoadFirstChildOf(CreateThreeLevelTreeWithLowerLevelFullReturnRootKey());
    auto node = dynamic_pointer_move<DataInnerNode>(_node);
    data->FillInto(node.get());
    return make_unique<DataTree>(&nodeStore, std::move(node));
  }

  unique_ptr<DataNode> LoadFirstChildOf(const Key &key) {
    auto root = LoadInnerNode(key);
    return nodeStore.load(root->getChild(0)->key());
  }

  unique_ptr<DataLeafNode> LoadFirstLeafOf(const Key &key) {
    auto root = LoadInnerNode(key);
    return LoadLeafNode(root->getChild(0)->key());
  }

  unique_ptr<DataLeafNode> LoadTwoLevelFirstLeafOf(const Key &key) {
    auto root = LoadInnerNode(key);
    auto inner = LoadInnerNode(root->getChild(0)->key());
    return LoadLeafNode(inner->getChild(0)->key());
  }
};

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAFullTwoLevelTree) {
  TwoLevelDataFixture data(&nodeStore);
  auto tree = CreateFullTwoLevelTreeWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto node = LoadFirstChildOf(tree->key());
  data.EXPECT_DATA_CORRECT(*dynamic_pointer_move<DataInnerNode>(node));
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeLevelTreeWithLowerLevelFull) {
  TwoLevelDataFixture data(&nodeStore);
  auto tree = CreateThreeLevelTreeWithLowerLevelFullWithData(&data);
  tree->addDataLeaf();
  tree->flush();

  auto node = LoadFirstChildOf(tree->key());
  data.EXPECT_DATA_CORRECT(*dynamic_pointer_move<DataInnerNode>(node));
}

class DataTreeGrowingTest_DataStaysIntact_OneDataLeaf: public DataTreeGrowingTest_DataStaysIntact, public WithParamInterface<uint32_t> {
};
INSTANTIATE_TEST_CASE_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, Values(0, 1, DataLeafNode::MAX_STORED_BYTES-2, DataLeafNode::MAX_STORED_BYTES-1, DataLeafNode::MAX_STORED_BYTES));

TEST_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, GrowAOneNodeTree) {
  LeafDataFixture data(GetParam());
  auto tree = CreateLeafOnlyTreeWithData(data);
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadFirstLeafOf(tree->key());
  data.EXPECT_DATA_CORRECT(*leaf);
}

TEST_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, GrowATwoNodeTree) {
  LeafDataFixture data(GetParam());
  auto tree = CreateTwoNodeTreeWithData(data);
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadFirstLeafOf(tree->key());
  data.EXPECT_DATA_CORRECT(*leaf);
}

TEST_P(DataTreeGrowingTest_DataStaysIntact_OneDataLeaf, GrowAThreeNodeChainedTree) {
  LeafDataFixture data(GetParam());
  auto tree = CreateThreeNodeChainedTreeWithData(data);
  tree->addDataLeaf();
  tree->flush();

  auto leaf = LoadTwoLevelFirstLeafOf(tree->key());
  data.EXPECT_DATA_CORRECT(*leaf);
}
