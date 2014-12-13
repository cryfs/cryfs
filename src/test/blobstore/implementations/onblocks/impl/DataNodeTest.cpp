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
  unique_ptr<BlockStore> blockStore = make_unique<FakeBlockStore>();
};

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

TEST_F(DataNodeTest, CreateLeafNodeCreatesLeafNode) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  auto node = DataNode::createNewLeafNode(std::move(block.block));
  EXPECT_IS_PTR_TYPE(DataLeafNode, node.get());
}

TEST_F(DataNodeTest, CreateInnerNodeCreatesInnerNode) {
  auto leafblock = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  auto leaf = DataNode::createNewLeafNode(std::move(leafblock.block));

  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  auto node = DataNode::createNewInnerNode(std::move(block.block), leafblock.key, *leaf);
  EXPECT_IS_PTR_TYPE(DataInnerNode, node.get());
}

TEST_F(DataNodeTest, LeafNodeIsRecognizedAfterStoreAndLoad) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  Key key = block.key;
  {
    DataNode::createNewLeafNode(std::move(block.block));
  }

  auto loaded_node = DataNode::load(blockStore->load(key));

  EXPECT_IS_PTR_TYPE(DataLeafNode, loaded_node.get());
}

TEST_F(DataNodeTest, InnerNodeWithDepth1IsRecognizedAfterStoreAndLoad) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  Key key = block.key;
  {
    auto leafblock = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
    auto leaf = DataNode::createNewLeafNode(std::move(leafblock.block));
    DataNode::createNewInnerNode(std::move(block.block), leafblock.key, *leaf);
  }

  auto loaded_node = DataNode::load(blockStore->load(key));

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeTest, InnerNodeWithDepth2IsRecognizedAfterStoreAndLoad) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  Key key = block.key;
  {
    auto leafblock = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
    auto leaf = DataNode::createNewLeafNode(std::move(leafblock.block));
    auto inner1block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
    auto inner1 = DataNode::createNewInnerNode(std::move(inner1block.block), leafblock.key, *leaf);
    DataNode::createNewInnerNode(std::move(block.block), inner1block.key, *inner1);
  }

  auto loaded_node = DataNode::load(blockStore->load(key));

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeTest, DataNodeCrashesOnLoadIfDepthIsTooHigh) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  Key key = block.key;
  {
    DataNodeView view(std::move(block.block));
    *view.Depth() = 200u; // this is an invalid depth
  }

  auto loaded_block = blockStore->load(key);
  EXPECT_ANY_THROW(
    DataNode::load(std::move(loaded_block))
  );
}
