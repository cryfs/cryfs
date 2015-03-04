#include "testutils/DataTreeTest.h"

#include "../../../../implementations/onblocks/datanodestore/DataNodeStore.h"
#include "../../../../implementations/onblocks/datatreestore/DataTreeStore.h"
#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>

using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using blobstore::onblocks::datanodestore::DataNodeStore;
using std::make_unique;

using namespace blobstore::onblocks::datatreestore;

class DataTreeStoreTest: public DataTreeTest {
};

TEST_F(DataTreeStoreTest, CorrectKeyReturned) {
  Key key = treeStore.createNewTree()->key();
  auto tree = treeStore.load(key);
  EXPECT_EQ(key, tree->key());
}

TEST_F(DataTreeStoreTest, CreatedTreeIsLoadable) {
  auto key = treeStore.createNewTree()->key();
  auto loaded = treeStore.load(key);
  EXPECT_NE(nullptr, loaded.get());
}

TEST_F(DataTreeStoreTest, NewTreeIsLeafOnly) {
  auto tree = treeStore.createNewTree();

  EXPECT_IS_LEAF_NODE(tree->key());
}

TEST_F(DataTreeStoreTest, TreeIsNotLoadableAfterRemove) {
  Key key = treeStore.createNewTree()->key();
  auto tree = treeStore.load(key);
  EXPECT_NE(nullptr, tree.get());
  treeStore.remove(std::move(tree));
  EXPECT_EQ(nullptr, treeStore.load(key).get());
}

TEST_F(DataTreeStoreTest, RemovingTreeRemovesAllNodesOfTheTree) {
  auto key = CreateThreeLevelMinData()->key();
  auto tree1 = treeStore.load(key);
  auto tree2_key = treeStore.createNewTree()->key();

  treeStore.remove(std::move(tree1));

  //Check that the only remaining node is tree2
  EXPECT_EQ(1, nodeStore->numNodes());
  EXPECT_NE(nullptr, treeStore.load(tree2_key).get());
}
