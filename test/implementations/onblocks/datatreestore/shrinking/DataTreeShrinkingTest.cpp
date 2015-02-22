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

//TODO Test that blocks are actually deleted
