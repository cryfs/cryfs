#include <gtest/gtest.h>

#include "blockstore/implementations/inmemory/InMemoryBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlock.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "blobstore/implementations/onblocks/impl/DataNode.h"
#include "blobstore/implementations/onblocks/impl/DataLeafNode.h"
#include "blobstore/implementations/onblocks/impl/DataInnerNode.h"

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;
using std::string;

using blockstore::BlockStore;
using blockstore::inmemory::InMemoryBlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;

class DataNodeTest: public Test {
public:
  unique_ptr<BlockStore> blockStore = make_unique<InMemoryBlockStore>();
};

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

TEST_F(DataNodeTest, InitializeNewLeafNodeCreatesLeafNodeObject) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  string key = block.key;
  auto leafNode = DataNode::initializeNewLeafNode(std::move(block.block));

  EXPECT_IS_PTR_TYPE(DataLeafNode, leafNode.get());
}

TEST_F(DataNodeTest, InitializeNewInnerNodeCreatesInnerNodeObject) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  string key = block.key;
  auto innerNode = DataNode::initializeNewInnerNode(std::move(block.block));

  EXPECT_IS_PTR_TYPE(DataInnerNode, innerNode.get());
}

TEST_F(DataNodeTest, LeafNodeIsRecognizedAfterStoreAndLoad) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  string key = block.key;
  auto node = DataNode::initializeNewLeafNode(std::move(block.block));
  node->flush();

  auto loaded_node = DataNode::load(blockStore->load(key));

  EXPECT_IS_PTR_TYPE(DataLeafNode, loaded_node.get());
}

TEST_F(DataNodeTest, InnerNodeIsRecognizedAfterStoreAndLoad) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  string key = block.key;
  auto node = DataNode::initializeNewInnerNode(std::move(block.block));
  node->flush();

  auto loaded_node = DataNode::load(blockStore->load(key));

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeTest, DataNodeCrashesOnLoadIfMagicNumberIsWrong) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  string key = block.key;
  DataNode::NodeHeader* header = (DataNode::NodeHeader*)block.block->data();
  header->magicNumber = 0xFF; // this is an invalid magic number

  EXPECT_ANY_THROW(
    DataNode::load(std::move(block.block))
  );
}
