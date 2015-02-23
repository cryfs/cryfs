#include "testutils/DataTreeShrinkingTest.h"

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

TEST_F(DataTreeShrinkingTest, ShrinkingALeafOnlyTreeCrashes) {
  Key key = CreateLeafOnlyTree()->key();
  auto tree = make_unique<DataTree>(&nodeStore, nodeStore.load(key));
  EXPECT_DEATH(tree->removeLastDataLeaf(), "");
}

TEST_F(DataTreeShrinkingTest, ShrinkATwoLeafTree_FlushingWorks) {
  //Tests that after calling flush(), the complete grown tree structure is written to the blockstore
  auto tree = CreateTwoLeafTree();
  tree->removeLastDataLeaf();
  tree->flush();

  EXPECT_IS_LEAF_NODE(tree->key());
}

TEST_F(DataTreeShrinkingTest, ShrinkATwoLeafTree_LastLeafBlockIsDeleted) {
  auto tree = CreateTwoLeafTree();
  tree->flush();
  auto lastChildKey = LoadInnerNode(tree->key())->getChild(1)->key();

  tree->removeLastDataLeaf();
  EXPECT_EQ(nullptr, nodeStore.load(lastChildKey));
}

TEST_F(DataTreeShrinkingTest, ShrinkATwoLeafTree_IntermediateBlocksAreDeleted) {
  auto tree = CreateTwoLeafTree();
  tree->flush();
  auto firstChildKey = LoadInnerNode(tree->key())->getChild(0)->key();

  tree->removeLastDataLeaf();
  EXPECT_EQ(nullptr, nodeStore.load(firstChildKey));
}

//TODO Test Shrinking full trees down to 1-leaf-tree
