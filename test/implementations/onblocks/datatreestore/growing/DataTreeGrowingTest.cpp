#include "testutils/DataTreeGrowingTest.h"

#include "../../../../testutils/DataBlockFixture.h"

using ::testing::WithParamInterface;
using ::testing::Values;

using std::unique_ptr;
using std::make_unique;

using cpputils::dynamic_pointer_move;
using namespace blobstore::onblocks::datatreestore;

using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blockstore::Key;

TEST_F(DataTreeGrowingTest, GrowAOneNodeTree_FlushingWorks) {
  //Tests that after calling flush(), the complete grown tree structure is written to the blockstore
  auto tree = CreateLeafOnlyTree();
  tree->addDataLeaf();
  tree->flush();

  EXPECT_INNER_NODE_NUMBER_OF_LEAVES_IS(2, tree->key());
}

//TODO Build-up test cases (build a leaf tree, add N leaves and check end state. End states for example FullTwoLevelTree, FullThreeLevelTree)
