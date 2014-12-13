#include <blobstore/implementations/onblocks/impl/DataNodeStore.h>
#include <gtest/gtest.h>

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlock.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "blobstore/implementations/onblocks/impl/DataNode.h"
#include "blobstore/implementations/onblocks/impl/DataLeafNode.h"
#include "blobstore/implementations/onblocks/impl/DataInnerNode.h"

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;
using std::string;

using blockstore::BlockStore;
using blockstore::testfake::FakeBlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;

class DataNodeTest: public Test {
public:
  unique_ptr<BlockStore> _blockStore = make_unique<FakeBlockStore>();
  BlockStore *blockStore = _blockStore.get();
  unique_ptr<DataNodeStore> nodeStore = make_unique<DataNodeStore>(std::move(_blockStore));
};

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

TEST_F(DataNodeTest, CreateLeafNodeCreatesLeafNode) {
  auto node = nodeStore->createNewLeafNode();
  EXPECT_IS_PTR_TYPE(DataLeafNode, node.get());
}

TEST_F(DataNodeTest, CreateInnerNodeCreatesInnerNode) {
  auto leaf = nodeStore->createNewLeafNode();

  auto node = nodeStore->createNewInnerNode(*leaf);
  EXPECT_IS_PTR_TYPE(DataInnerNode, node.get());
}

TEST_F(DataNodeTest, LeafNodeIsRecognizedAfterStoreAndLoad) {
  Key key = nodeStore->createNewLeafNode()->key();

  auto loaded_node = nodeStore->load(key);

  EXPECT_IS_PTR_TYPE(DataLeafNode, loaded_node.get());
}

TEST_F(DataNodeTest, InnerNodeWithDepth1IsRecognizedAfterStoreAndLoad) {
  auto leaf = nodeStore->createNewLeafNode();
  Key key = nodeStore->createNewInnerNode(*leaf)->key();

  auto loaded_node = nodeStore->load(key);

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeTest, InnerNodeWithDepth2IsRecognizedAfterStoreAndLoad) {
  auto leaf = nodeStore->createNewLeafNode();
  auto inner = nodeStore->createNewInnerNode(*leaf);
  Key key = nodeStore->createNewInnerNode(*inner)->key();

  auto loaded_node = nodeStore->load(key);

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeTest, DataNodeCrashesOnLoadIfDepthIsTooHigh) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  Key key = block.key;
  {
    DataNodeView view(std::move(block.block));
    *view.Depth() = DataNodeStore::MAX_DEPTH + 1;
  }

  EXPECT_ANY_THROW(
    nodeStore->load(key)
  );
}
