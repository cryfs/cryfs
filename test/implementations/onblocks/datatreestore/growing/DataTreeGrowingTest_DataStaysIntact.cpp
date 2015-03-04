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
  unique_ptr<DataTree> TreeWithData(unique_ptr<DataNode> root, TwoLevelDataFixture *data) {
    data->FillInto(root.get());
    Key key = root->key();
    root.reset();
    return treeStore.load(key);
  }

  void TestDataStaysIntactOnGrowing(unique_ptr<DataNode> root, TwoLevelDataFixture *data) {
    uint32_t numLeaves = countLeaves(root.get());
    auto tree = TreeWithData(std::move(root), data);
    tree->addDataLeaf();
    tree->flush();

    data->EXPECT_DATA_CORRECT(nodeStore->load(tree->key()).get(), numLeaves);
  }

  uint32_t countLeaves(DataNode *node) {
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(node);
    if (inner == nullptr) {
      return 1;
    }
    uint32_t result = 0;
    for(int i = 0; i < inner->numChildren(); ++i) {
      result += countLeaves(nodeStore->load(inner->getChild(i)->key()).get());
    }
    return result;
  }
};

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAFullTwoLevelTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Full);
  TestDataStaysIntactOnGrowing(CreateFullTwoLevel(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAFullTwoLevelTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Random);
  TestDataStaysIntactOnGrowing(CreateFullTwoLevel(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeLevelTreeWithLowerLevelFull_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Full);
  auto node = CreateInner({CreateFullTwoLevel()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeLevelTreeWithLowerLevelFull_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Random);
  auto node = CreateInner({CreateFullTwoLevel()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAOneNodeTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Full);
  TestDataStaysIntactOnGrowing(CreateLeaf(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAOneNodeTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Random);
  TestDataStaysIntactOnGrowing(CreateLeaf(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowATwoNodeTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Full);
  auto node = CreateInner({CreateLeaf()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowATwoNodeTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Random);
  auto node = CreateInner({CreateLeaf()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeNodeChainedTree_FullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Full);
  auto node = CreateInner({CreateInner({CreateLeaf()})});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeNodeChainedTree_NonFullLeaves) {
  TwoLevelDataFixture data(nodeStore, TwoLevelDataFixture::SizePolicy::Random);
  auto node = CreateInner({CreateInner({CreateLeaf()})});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}
