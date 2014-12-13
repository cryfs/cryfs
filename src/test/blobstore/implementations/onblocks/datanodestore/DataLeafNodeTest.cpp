#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>
#include <gtest/gtest.h>

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlock.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "test/testutils/DataBlockFixture.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Combine;
using std::unique_ptr;
using std::make_unique;
using std::string;

using blockstore::BlockStore;
using blockstore::BlockWithKey;
using blockstore::Data;
using blockstore::Key;
using blockstore::testfake::FakeBlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

class DataLeafNodeTest: public Test {
public:

  DataLeafNodeTest():
    ZEROES(DataLeafNode::MAX_STORED_BYTES),
    randomData(DataLeafNode::MAX_STORED_BYTES),
    _blockStore(make_unique<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique<DataNodeStore>(std::move(_blockStore))),
    leaf(nodeStore->createNewLeafNode()) {

    ZEROES.FillWithZeroes();

    DataBlockFixture dataFixture(DataLeafNode::MAX_STORED_BYTES);

    std::memcpy(randomData.data(), dataFixture.data(), randomData.size());
  }

  Key WriteDataToNewLeafBlockAndReturnKey() {
    auto newleaf = nodeStore->createNewLeafNode();
    newleaf->resize(randomData.size());
    newleaf->write(0, randomData.size(), randomData);
    return newleaf->key();
  }

  void FillLeafBlockWithData() {
    leaf->resize(randomData.size());
    leaf->write(0, randomData.size(), randomData);
  }

  void ReadDataFromLoadedLeafBlock(Key key, Data *data) {
    auto leaf = nodeStore->load(key);
    EXPECT_IS_PTR_TYPE(DataLeafNode, leaf.get());
    leaf->read(0, data->size(), data);
  }

  void ResizeLeaf(const Key &key, size_t size) {
    auto leaf = nodeStore->load(key);
    EXPECT_IS_PTR_TYPE(DataLeafNode, leaf.get());
    leaf->resize(size);
  }

  Data ZEROES;
  Data randomData;
  unique_ptr<BlockStore> _blockStore;
  BlockStore *blockStore;
  unique_ptr<DataNodeStore> nodeStore;
  unique_ptr<DataLeafNode> leaf;
};

TEST_F(DataLeafNodeTest, InitializesCorrectly) {
  leaf->InitializeNewNode();
  EXPECT_EQ(0u, leaf->numBytesInThisNode());
}

TEST_F(DataLeafNodeTest, ReinitializesCorrectly) {
  leaf->resize(5);
  leaf->InitializeNewNode();
  EXPECT_EQ(0u, leaf->numBytesInThisNode());
}

TEST_F(DataLeafNodeTest, ReadWrittenDataImmediately) {
  leaf->resize(randomData.size());
  leaf->write(0, randomData.size(), randomData);

  Data read(DataLeafNode::MAX_STORED_BYTES);
  leaf->read(0, read.size(), &read);
  EXPECT_EQ(0, std::memcmp(randomData.data(), read.data(), randomData.size()));
}

TEST_F(DataLeafNodeTest, ReadWrittenDataAfterReloadingBLock) {
  Key key = WriteDataToNewLeafBlockAndReturnKey();

  Data data(DataLeafNode::MAX_STORED_BYTES);
  ReadDataFromLoadedLeafBlock(key, &data);

  EXPECT_EQ(0, std::memcmp(randomData.data(), data.data(), randomData.size()));
}

TEST_F(DataLeafNodeTest, NewLeafNodeHasSizeZero) {
  EXPECT_EQ(0u, leaf->numBytesInThisNode());
}

TEST_F(DataLeafNodeTest, NewLeafNodeHasSizeZero_AfterLoading) {
  Key key = nodeStore->createNewLeafNode()->key();
  auto leaf = nodeStore->load(key);

  EXPECT_EQ(0u, leaf->numBytesInThisNode());
}

class DataLeafNodeSizeTest: public DataLeafNodeTest, public WithParamInterface<unsigned int> {
public:
  Key CreateLeafResizeItAndReturnKey() {
    auto leaf = nodeStore->createNewLeafNode();
    leaf->resize(GetParam());
    return leaf->key();
  }
};
INSTANTIATE_TEST_CASE_P(DataLeafNodeSizeTest, DataLeafNodeSizeTest, Values(0, 1, 5, 16, 32, 512, DataLeafNode::MAX_STORED_BYTES));

TEST_P(DataLeafNodeSizeTest, ResizeNode_ReadSizeImmediately) {
  leaf->resize(GetParam());
  EXPECT_EQ(GetParam(), leaf->numBytesInThisNode());
}

TEST_P(DataLeafNodeSizeTest, ResizeNode_ReadSizeAfterLoading) {
  Key key = CreateLeafResizeItAndReturnKey();

  auto leaf = nodeStore->load(key);
  EXPECT_EQ(GetParam(), leaf->numBytesInThisNode());
}

TEST_F(DataLeafNodeTest, SpaceIsZeroFilledWhenGrowing) {
  leaf->resize(randomData.size());

  Data read(randomData.size());
  leaf->read(0, read.size(), &read);
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), read.data(), read.size()));
}

TEST_F(DataLeafNodeTest, SpaceGetsZeroFilledWhenShrinkingAndRegrowing) {
  FillLeafBlockWithData();
  // resize it smaller and then back to original size
  uint32_t smaller_size = randomData.size() - 100;
  leaf->resize(smaller_size);
  leaf->resize(randomData.size());

  //Check that the space was filled with zeroes
  Data read(100);
  leaf->read(smaller_size, read.size(), &read);
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), read.data(), read.size()));
}

TEST_F(DataLeafNodeTest, DataGetsZeroFilledWhenShrinking) {
  Key key = WriteDataToNewLeafBlockAndReturnKey();
  uint32_t smaller_size = randomData.size() - 100;
  {
    //At first, we expect there to be random data in the underlying data block
    auto block = blockStore->load(key);
    EXPECT_EQ(0, std::memcmp((char*)randomData.data()+smaller_size, (uint8_t*)block->data()+DataNodeView::HEADERSIZE_BYTES+smaller_size, 100));
  }

  //After shrinking, we expect there to be zeroes in the underlying data block
  ResizeLeaf(key, smaller_size);
  {
    auto block = blockStore->load(leaf->key());
    EXPECT_EQ(0, std::memcmp(ZEROES.data(), (uint8_t*)block->data()+DataNodeView::HEADERSIZE_BYTES+smaller_size, 100));
  }
}

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

