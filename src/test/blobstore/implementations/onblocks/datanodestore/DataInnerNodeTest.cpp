#include <gtest/gtest.h>

#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlock.h"

#include <memory>

using ::testing::Test;

using blockstore::Key;
using blockstore::testfake::FakeBlockStore;
using blockstore::BlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

using std::unique_ptr;
using std::make_unique;

class DataInnerNodeTest: public Test {
public:
  DataInnerNodeTest() :
    _blockStore(make_unique<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique<DataNodeStore>(std::move(_blockStore))),
    leaf(nodeStore->createNewLeafNode()),
    node(nodeStore->createNewInnerNode(*leaf)) {
  }

  unique_ptr<BlockStore> _blockStore;
  BlockStore *blockStore;
  unique_ptr<DataNodeStore> nodeStore;
  unique_ptr<DataLeafNode> leaf;
  unique_ptr<DataInnerNode> node;
};

TEST_F(DataInnerNodeTest, InitializesCorrectly) {
  node->InitializeNewNode(*leaf);
  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, ReinitializesCorrectly) {
  node->InitializeNewNode(*leaf);
  auto leaf2 = nodeStore->createNewLeafNode();
  node->addChild(*leaf2);
  node->InitializeNewNode(*leaf);

  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
}

TEST_F(DataInnerNodeTest, AddingASecondLeaf) {
  auto leaf2 = nodeStore->createNewLeafNode();
  node->addChild(*leaf2);

  EXPECT_EQ(2u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
  EXPECT_EQ(leaf2->key(), node->getChild(1)->key());
}
