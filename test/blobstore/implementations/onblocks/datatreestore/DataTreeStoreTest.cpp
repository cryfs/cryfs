#include "testutils/DataTreeTest.h"

#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTreeStore.h"
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using blobstore::onblocks::datanodestore::DataNodeStore;
using boost::none;

using namespace blobstore::onblocks::datatreestore;

class DataTreeStoreTest: public DataTreeTest {
};

TEST_F(DataTreeStoreTest, CorrectKeyReturned) {
  Key key = treeStore.createNewTree()->key();
  auto tree = treeStore.load(key).value();
  EXPECT_EQ(key, tree->key());
}

TEST_F(DataTreeStoreTest, CreatedTreeIsLoadable) {
  auto key = treeStore.createNewTree()->key();
  auto loaded = treeStore.load(key);
  EXPECT_NE(none, loaded);
}

TEST_F(DataTreeStoreTest, NewTreeIsLeafOnly) {
  auto tree = treeStore.createNewTree();

  EXPECT_IS_LEAF_NODE(tree->key());
}

TEST_F(DataTreeStoreTest, TreeIsNotLoadableAfterRemove) {
  Key key = treeStore.createNewTree()->key();
  auto tree = treeStore.load(key);
  EXPECT_NE(none, tree);
  treeStore.remove(std::move(*tree));
  EXPECT_EQ(none, treeStore.load(key));
}

TEST_F(DataTreeStoreTest, RemovingTreeRemovesAllNodesOfTheTree) {
  auto key = CreateThreeLevelMinData()->key();
  auto tree1 = treeStore.load(key).value();
  auto tree2_key = treeStore.createNewTree()->key();

  treeStore.remove(std::move(tree1));

  //Check that the only remaining node is tree2
  EXPECT_EQ(1u, nodeStore->numNodes());
  EXPECT_NE(none, treeStore.load(tree2_key));
}
