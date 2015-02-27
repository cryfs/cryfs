#include "../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataInnerNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataNodeStore.h"
#include <google/gtest/gtest.h>

#include "messmer/cpp-utils/pointer.h"

#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"
#include "messmer/blockstore/implementations/testfake/FakeBlock.h"
#include "messmer/blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "../../../testutils/DataBlockFixture.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Combine;
using std::unique_ptr;
using std::make_unique;
using std::string;

using cpputils::dynamic_pointer_move;

using blockstore::BlockStore;
using blockstore::Data;
using blockstore::Key;
using blockstore::testfake::FakeBlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

class DataLeafNodeTest: public Test {
public:

  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;

  DataLeafNodeTest():
    _blockStore(make_unique<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES)),
    randomData(nodeStore->layout().maxBytesPerLeaf()),
    ZEROES(nodeStore->layout().maxBytesPerLeaf()),
    leaf(nodeStore->createNewLeafNode()) {

    ZEROES.FillWithZeroes();

    DataBlockFixture dataFixture(nodeStore->layout().maxBytesPerLeaf());

    std::memcpy(randomData.data(), dataFixture.data(), randomData.size());
  }

  Key WriteDataToNewLeafBlockAndReturnKey() {
    auto newleaf = nodeStore->createNewLeafNode();
    newleaf->resize(randomData.size());
    std::memcpy(newleaf->data(), randomData.data(), randomData.size());
    return newleaf->key();
  }

  void FillLeafBlockWithData() {
    FillLeafBlockWithData(leaf.get());
  }

  void FillLeafBlockWithData(DataLeafNode *leaf_to_fill) {
    leaf_to_fill->resize(randomData.size());
    std::memcpy(leaf_to_fill->data(), randomData.data(), randomData.size());
  }

  unique_ptr<DataLeafNode> LoadLeafNode(const Key &key) {
    auto leaf = nodeStore->load(key);
    return dynamic_pointer_move<DataLeafNode>(leaf);
  }

  void ResizeLeaf(const Key &key, size_t size) {
    auto leaf = LoadLeafNode(key);
    EXPECT_IS_PTR_TYPE(DataLeafNode, leaf.get());
    leaf->resize(size);
  }

  Key CreateLeafWithDataConvertItToInnerNodeAndReturnKey() {
    auto leaf = nodeStore->createNewLeafNode();
    FillLeafBlockWithData(leaf.get());
    auto child = nodeStore->createNewLeafNode();
    unique_ptr<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(leaf), *child);
    return converted->key();
  }

  unique_ptr<DataLeafNode> CopyLeafNode(const DataLeafNode &node) {
    auto copied = nodeStore->createNewNodeAsCopyFrom(node);
    return dynamic_pointer_move<DataLeafNode>(copied);
  }

  Key InitializeLeafGrowAndReturnKey() {
    auto leaf = DataLeafNode::InitializeNewNode(blockStore->create(BLOCKSIZE_BYTES));
    leaf->resize(5);
    return leaf->key();
  }

  unique_ptr<BlockStore> _blockStore;
  BlockStore *blockStore;
  unique_ptr<DataNodeStore> nodeStore;
  Data ZEROES;
  Data randomData;
  unique_ptr<DataLeafNode> leaf;
};

constexpr uint32_t DataLeafNodeTest::BLOCKSIZE_BYTES;

TEST_F(DataLeafNodeTest, CorrectKeyReturnedAfterInitialization) {
  auto block = blockStore->create(BLOCKSIZE_BYTES);
  Key key = block->key();
  auto node = DataLeafNode::InitializeNewNode(std::move(block));
  EXPECT_EQ(key, node->key());
}

TEST_F(DataLeafNodeTest, CorrectKeyReturnedAfterLoading) {
  auto block = blockStore->create(BLOCKSIZE_BYTES);
  Key key = block->key();
  DataLeafNode::InitializeNewNode(std::move(block));

  auto loaded = nodeStore->load(key);
  EXPECT_EQ(key, loaded->key());
}

TEST_F(DataLeafNodeTest, InitializesCorrectly) {
  auto leaf = DataLeafNode::InitializeNewNode(blockStore->create(BLOCKSIZE_BYTES));
  EXPECT_EQ(0u, leaf->numBytes());
}

TEST_F(DataLeafNodeTest, ReinitializesCorrectly) {
  auto key = InitializeLeafGrowAndReturnKey();
  auto leaf = DataLeafNode::InitializeNewNode(blockStore->load(key));
  EXPECT_EQ(0u, leaf->numBytes());
}

TEST_F(DataLeafNodeTest, ReadWrittenDataAfterReloadingBlock) {
  Key key = WriteDataToNewLeafBlockAndReturnKey();

  auto loaded = LoadLeafNode(key);

  EXPECT_EQ(randomData.size(), loaded->numBytes());
  EXPECT_EQ(0, std::memcmp(randomData.data(), loaded->data(), randomData.size()));
}

TEST_F(DataLeafNodeTest, NewLeafNodeHasSizeZero) {
  EXPECT_EQ(0u, leaf->numBytes());
}

TEST_F(DataLeafNodeTest, NewLeafNodeHasSizeZero_AfterLoading) {
  Key key = nodeStore->createNewLeafNode()->key();
  auto leaf = LoadLeafNode(key);

  EXPECT_EQ(0u, leaf->numBytes());
}

class DataLeafNodeSizeTest: public DataLeafNodeTest, public WithParamInterface<unsigned int> {
public:
  Key CreateLeafResizeItAndReturnKey() {
    auto leaf = nodeStore->createNewLeafNode();
    leaf->resize(GetParam());
    return leaf->key();
  }
};
INSTANTIATE_TEST_CASE_P(DataLeafNodeSizeTest, DataLeafNodeSizeTest, Values(0, 1, 5, 16, 32, 512, DataNodeLayout(DataLeafNodeTest::BLOCKSIZE_BYTES).maxBytesPerLeaf()));

TEST_P(DataLeafNodeSizeTest, ResizeNode_ReadSizeImmediately) {
  leaf->resize(GetParam());
  EXPECT_EQ(GetParam(), leaf->numBytes());
}

TEST_P(DataLeafNodeSizeTest, ResizeNode_ReadSizeAfterLoading) {
  Key key = CreateLeafResizeItAndReturnKey();

  auto leaf = LoadLeafNode(key);
  EXPECT_EQ(GetParam(), leaf->numBytes());
}

TEST_F(DataLeafNodeTest, SpaceIsZeroFilledWhenGrowing) {
  leaf->resize(randomData.size());
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), leaf->data(), randomData.size()));
}

TEST_F(DataLeafNodeTest, SpaceGetsZeroFilledWhenShrinkingAndRegrowing) {
  FillLeafBlockWithData();
  // resize it smaller and then back to original size
  uint32_t smaller_size = randomData.size() - 100;
  leaf->resize(smaller_size);
  leaf->resize(randomData.size());

  //Check that the space was filled with zeroes
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), ((uint8_t*)leaf->data())+smaller_size, 100));
}

TEST_F(DataLeafNodeTest, DataGetsZeroFilledWhenShrinking) {
  Key key = WriteDataToNewLeafBlockAndReturnKey();
  uint32_t smaller_size = randomData.size() - 100;
  {
    //At first, we expect there to be random data in the underlying data block
    auto block = blockStore->load(key);
    EXPECT_EQ(0, std::memcmp((char*)randomData.data()+smaller_size, (uint8_t*)block->data()+DataNodeLayout::HEADERSIZE_BYTES+smaller_size, 100));
  }

  //After shrinking, we expect there to be zeroes in the underlying data block
  ResizeLeaf(key, smaller_size);
  {
    auto block = blockStore->load(key);
    EXPECT_EQ(0, std::memcmp(ZEROES.data(), (uint8_t*)block->data()+DataNodeLayout::HEADERSIZE_BYTES+smaller_size, 100));
  }
}

TEST_F(DataLeafNodeTest, ShrinkingDoesntDestroyValidDataRegion) {
  FillLeafBlockWithData();
  uint32_t smaller_size = randomData.size() - 100;
  leaf->resize(smaller_size);

  //Check that the remaining data region is unchanged
  EXPECT_EQ(0, std::memcmp(randomData.data(), leaf->data(), smaller_size));
}

TEST_F(DataLeafNodeTest, ConvertToInternalNode) {
  auto child = nodeStore->createNewLeafNode();
  Key leaf_key = leaf->key();
  unique_ptr<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(leaf), *child);

  EXPECT_EQ(1u, converted->numChildren());
  EXPECT_EQ(child->key(), converted->getChild(0)->key());
  EXPECT_EQ(leaf_key, converted->key());
}

TEST_F(DataLeafNodeTest, ConvertToInternalNodeZeroesOutChildrenRegion) {
  Key key = CreateLeafWithDataConvertItToInnerNodeAndReturnKey();

  auto block = blockStore->load(key);
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), (uint8_t*)block->data()+DataNodeLayout::HEADERSIZE_BYTES+sizeof(DataInnerNode::ChildEntry), nodeStore->layout().maxBytesPerLeaf()-sizeof(DataInnerNode::ChildEntry)));
}

TEST_F(DataLeafNodeTest, CopyingCreatesANewLeaf) {
  auto copied = CopyLeafNode(*leaf);
  EXPECT_NE(leaf->key(), copied->key());
}

TEST_F(DataLeafNodeTest, CopyEmptyLeaf) {
  auto copied = CopyLeafNode(*leaf);
  EXPECT_EQ(leaf->numBytes(), copied->numBytes());
}

TEST_F(DataLeafNodeTest, CopyDataLeaf) {
  FillLeafBlockWithData();
  auto copied = CopyLeafNode(*leaf);

  EXPECT_EQ(leaf->numBytes(), copied->numBytes());
  EXPECT_EQ(0, std::memcmp(leaf->data(), copied->data(), leaf->numBytes()));
  EXPECT_NE(leaf->data(), copied->data());
}

/* TODO
 * The following test cases test reading/writing part of a leaf. This doesn't make much sense,
 * since the new leaf abstraction doesn't offer read()/write() anymore, but direct data pointer access.
 * However, these test cases might make sense wherever the read()/write() for a leaf will be implemented.
 * In case they're not needed then, delete them.

struct DataRange {
  DataRange(size_t leafsize_, off_t offset_, size_t count_): leafsize(leafsize_), offset(offset_), count(count_) {}
  size_t leafsize;
  off_t offset;
  size_t count;
};

class DataLeafNodeDataTest: public DataLeafNodeTest, public WithParamInterface<DataRange> {
public:
  Data foregroundData;
  Data backgroundData;

  DataLeafNodeDataTest(): foregroundData(GetParam().count), backgroundData(GetParam().leafsize) {
    DataBlockFixture _foregroundData(GetParam().count);
    DataBlockFixture _backgroundData(GetParam().leafsize);
    std::memcpy(foregroundData.data(), _foregroundData.data(), foregroundData.size());
    std::memcpy(backgroundData.data(), _backgroundData.data(), backgroundData.size());
  }

  void EXPECT_DATA_EQ(const Data &expected, const Data &actual) {
    EXPECT_EQ(expected.size(), actual.size());
    EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), expected.size()));
  }

  Key CreateLeafWriteToItAndReturnKey(const Data &to_write) {
    auto newleaf = nodeStore->createNewLeafNode();

    newleaf->resize(GetParam().leafsize);
    newleaf->write(GetParam().offset, GetParam().count, to_write);
    return newleaf->key();
  }

  void EXPECT_DATA_READS_AS(const Data &expected, const DataNode &leaf, off_t offset, size_t count) {
    Data read(count);
    leaf.read(offset, count, &read);
    EXPECT_DATA_EQ(expected, read);
  }

  void EXPECT_DATA_READS_AS_OUTSIDE_OF(const Data &expected, const DataNode &leaf, off_t start, size_t count) {
    Data begin(start);
    Data end(GetParam().leafsize - count - start);

    std::memcpy(begin.data(), expected.data(), start);
    std::memcpy(end.data(), (uint8_t*)expected.data()+start+count, end.size());

    EXPECT_DATA_READS_AS(begin, leaf, 0, start);
    EXPECT_DATA_READS_AS(end, leaf, start + count, end.size());
  }

  void EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(const DataNode &leaf, off_t start, size_t count) {
    Data ZEROES(GetParam().leafsize);
    ZEROES.FillWithZeroes();
    EXPECT_DATA_READS_AS_OUTSIDE_OF(ZEROES, leaf, start, count);
  }
};
INSTANTIATE_TEST_CASE_P(DataLeafNodeDataTest, DataLeafNodeDataTest, Values(
  DataRange(DataLeafNode::MAX_STORED_BYTES,     0,   DataLeafNode::MAX_STORED_BYTES),     // full size leaf, access beginning to end
  DataRange(DataLeafNode::MAX_STORED_BYTES,     100, DataLeafNode::MAX_STORED_BYTES-200), // full size leaf, access middle to middle
  DataRange(DataLeafNode::MAX_STORED_BYTES,     0,   DataLeafNode::MAX_STORED_BYTES-100), // full size leaf, access beginning to middle
  DataRange(DataLeafNode::MAX_STORED_BYTES,     100, DataLeafNode::MAX_STORED_BYTES-100), // full size leaf, access middle to end
  DataRange(DataLeafNode::MAX_STORED_BYTES-100, 0,   DataLeafNode::MAX_STORED_BYTES-100), // non-full size leaf, access beginning to end
  DataRange(DataLeafNode::MAX_STORED_BYTES-100, 100, DataLeafNode::MAX_STORED_BYTES-300), // non-full size leaf, access middle to middle
  DataRange(DataLeafNode::MAX_STORED_BYTES-100, 0,   DataLeafNode::MAX_STORED_BYTES-200), // non-full size leaf, access beginning to middle
  DataRange(DataLeafNode::MAX_STORED_BYTES-100, 100, DataLeafNode::MAX_STORED_BYTES-200)  // non-full size leaf, access middle to end
));

TEST_P(DataLeafNodeDataTest, WriteAndReadImmediately) {
  leaf->resize(GetParam().leafsize);
  leaf->write(GetParam().offset, GetParam().count, this->foregroundData);

  EXPECT_DATA_READS_AS(this->foregroundData, *leaf, GetParam().offset, GetParam().count);
  EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*leaf, GetParam().offset, GetParam().count);
}

TEST_P(DataLeafNodeDataTest, WriteAndReadAfterLoading) {
  Key key = CreateLeafWriteToItAndReturnKey(this->foregroundData);

  auto loaded_leaf = nodeStore->load(key);
  EXPECT_DATA_READS_AS(this->foregroundData, *loaded_leaf, GetParam().offset, GetParam().count);
  EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*loaded_leaf, GetParam().offset, GetParam().count);
}

TEST_P(DataLeafNodeDataTest, OverwriteAndRead) {
  leaf->resize(GetParam().leafsize);
  leaf->write(0, GetParam().leafsize, this->backgroundData);
  leaf->write(GetParam().offset, GetParam().count, this->foregroundData);
  EXPECT_DATA_READS_AS(this->foregroundData, *leaf, GetParam().offset, GetParam().count);
  EXPECT_DATA_READS_AS_OUTSIDE_OF(this->backgroundData, *leaf, GetParam().offset, GetParam().count);
}
*/

