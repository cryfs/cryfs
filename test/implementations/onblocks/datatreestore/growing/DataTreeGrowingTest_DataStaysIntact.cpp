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
    return make_unique<DataTree>(&nodeStore, std::move(root));
  }

  void TestDataStaysIntactOnGrowing(unique_ptr<DataNode> root, TwoLevelDataFixture *data) {
    uint32_t numLeaves = countLeaves(root.get());
    auto tree = TreeWithData(std::move(root), data);
    tree->addDataLeaf();
    tree->flush();

    data->EXPECT_DATA_CORRECT(nodeStore.load(tree->key()).get(), numLeaves);
  }

  uint32_t countLeaves(DataNode *node) {
    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(node);
    if (inner == nullptr) {
      return 1;
    }
    uint32_t result = 0;
    for(int i = 0; i < inner->numChildren(); ++i) {
      result += countLeaves(nodeStore.load(inner->getChild(i)->key()).get());
    }
    return result;
  }
};

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAFullTwoLevelTree_FullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, true);
  TestDataStaysIntactOnGrowing(CreateFullTwoLevel(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAFullTwoLevelTree_NonFullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, false);
  TestDataStaysIntactOnGrowing(CreateFullTwoLevel(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeLevelTreeWithLowerLevelFull_FullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, true);
  auto node = CreateInner({CreateFullTwoLevel()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeLevelTreeWithLowerLevelFull_NonFullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, false);
  auto node = CreateInner({CreateFullTwoLevel()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAOneNodeTree_FullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, true);
  TestDataStaysIntactOnGrowing(CreateLeaf(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAOneNodeTree_NonFullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, false);
  TestDataStaysIntactOnGrowing(CreateLeaf(), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowATwoNodeTree_FullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, true);
  auto node = CreateInner({CreateLeaf()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowATwoNodeTree_NonFullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, false);
  auto node = CreateInner({CreateLeaf()});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeNodeChainedTree_FullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, true);
  auto node = CreateInner({CreateInner({CreateLeaf()})});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}

TEST_F(DataTreeGrowingTest_DataStaysIntact, GrowAThreeNodeChainedTree_NonFullLeaves) {
  TwoLevelDataFixture data(&nodeStore, 0, false);
  auto node = CreateInner({CreateInner({CreateLeaf()})});
  TestDataStaysIntactOnGrowing(std::move(node), &data);
}
