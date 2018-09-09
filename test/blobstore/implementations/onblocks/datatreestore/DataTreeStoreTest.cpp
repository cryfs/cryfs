#include "testutils/DataTreeTest.h"

#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTreeStore.h"
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using blockstore::BlockId;
using boost::none;

using namespace blobstore::onblocks::datatreestore;

class DataTreeStoreTest: public DataTreeTest {
};

TEST_F(DataTreeStoreTest, CorrectKeyReturned) {
  BlockId blockId = treeStore.createNewTree()->blockId();
  auto tree = treeStore.load(blockId).value();
  EXPECT_EQ(blockId, tree->blockId());
}

TEST_F(DataTreeStoreTest, CreatedTreeIsLoadable) {
  auto blockId = treeStore.createNewTree()->blockId();
  auto loaded = treeStore.load(blockId);
  EXPECT_NE(none, loaded);
}

TEST_F(DataTreeStoreTest, NewTreeIsLeafOnly) {
  auto tree = treeStore.createNewTree();

  EXPECT_IS_LEAF_NODE(tree->blockId());
}

TEST_F(DataTreeStoreTest, TreeIsNotLoadableAfterRemove_DeleteByTree) {
  BlockId blockId = treeStore.createNewTree()->blockId();
  auto tree = treeStore.load(blockId);
  EXPECT_NE(none, tree);
  treeStore.remove(std::move(*tree));
  EXPECT_EQ(none, treeStore.load(blockId));
}

TEST_F(DataTreeStoreTest, TreeIsNotLoadableAfterRemove_DeleteByKey) {
  BlockId blockId = treeStore.createNewTree()->blockId();
  treeStore.remove(blockId);
  EXPECT_EQ(none, treeStore.load(blockId));
}

TEST_F(DataTreeStoreTest, RemovingTreeRemovesAllNodesOfTheTree_DeleteByTree) {
  auto tree1_blockId = CreateThreeLevelMinData()->blockId();
  auto tree2_blockId = treeStore.createNewTree()->blockId();

  auto tree1 = treeStore.load(tree1_blockId).value();
  treeStore.remove(std::move(tree1));

  //Check that the only remaining node is tree2
  EXPECT_EQ(1u, nodeStore->numNodes());
  EXPECT_NE(none, treeStore.load(tree2_blockId));
}

TEST_F(DataTreeStoreTest, RemovingTreeRemovesAllNodesOfTheTree_DeleteByKey) {
  auto tree1_blockId = CreateThreeLevelMinData()->blockId();
  auto tree2_blockId = treeStore.createNewTree()->blockId();

  treeStore.remove(tree1_blockId);

  //Check that the only remaining node is tree2
  EXPECT_EQ(1u, nodeStore->numNodes());
  EXPECT_NE(none, treeStore.load(tree2_blockId));
}
