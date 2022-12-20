#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include <gtest/gtest.h>

#include <cpp-utils/pointer/cast.h>

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <blockstore/implementations/testfake/FakeBlock.h>
#include <cpp-utils/data/DataFixture.h>

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::string;
using cpputils::DataFixture;
using cpputils::deserialize;

//TODO Split into multiple files

using cpputils::dynamic_pointer_move;

using blockstore::BlockStore;
using cpputils::Data;
using blockstore::BlockId;
using blockstore::testfake::FakeBlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

namespace {

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

class DataLeafNodeTest: public Test {
public:

  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;
  static constexpr DataNodeLayout LAYOUT = DataNodeLayout(BLOCKSIZE_BYTES);

  DataLeafNodeTest():
    _blockStore(make_unique_ref<FakeBlockStore>()),
    blockStore(_blockStore.get()),
    nodeStore(make_unique_ref<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES)),
    ZEROES(nodeStore->layout().maxBytesPerLeaf()),
    randomData(nodeStore->layout().maxBytesPerLeaf()),
    leaf(nodeStore->createNewLeafNode(Data(0))) {

    ZEROES.FillWithZeroes();

    Data dataFixture(DataFixture::generate(nodeStore->layout().maxBytesPerLeaf()));

    std::memcpy(randomData.data(), dataFixture.data(), randomData.size());
  }

  Data loadData(const DataLeafNode &leaf) {
    Data data(leaf.numBytes());
    leaf.read(data.data(), 0, leaf.numBytes());
    return data;
  }

  BlockId WriteDataToNewLeafBlockAndReturnKey() {
    auto newleaf = nodeStore->createNewLeafNode(Data(0));
    newleaf->resize(randomData.size());
    newleaf->write(randomData.data(), 0, randomData.size());
    return newleaf->blockId();
  }

  void FillLeafBlockWithData() {
    FillLeafBlockWithData(leaf.get());
  }

  void FillLeafBlockWithData(DataLeafNode *leaf_to_fill) {
    leaf_to_fill->resize(randomData.size());
    leaf_to_fill->write(randomData.data(), 0, randomData.size());
  }

  unique_ref<DataLeafNode> LoadLeafNode(const BlockId &blockId) {
    auto leaf = nodeStore->load(blockId).value();
    return dynamic_pointer_move<DataLeafNode>(leaf).value();
  }

  void ResizeLeaf(const BlockId &blockId, size_t size) {
    auto leaf = LoadLeafNode(blockId);
    EXPECT_IS_PTR_TYPE(DataLeafNode, leaf.get());
    leaf->resize(size);
  }

  BlockId CreateLeafWithDataConvertItToInnerNodeAndReturnKey() {
    auto leaf = nodeStore->createNewLeafNode(Data(0));
    FillLeafBlockWithData(leaf.get());
    auto child = nodeStore->createNewLeafNode(Data(0));
    unique_ref<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(leaf), LAYOUT, *child);
    return converted->blockId();
  }

  unique_ref<DataLeafNode> CopyLeafNode(const DataLeafNode &node) {
    auto copied = nodeStore->createNewNodeAsCopyFrom(node);
    return dynamic_pointer_move<DataLeafNode>(copied).value();
  }

  BlockId InitializeLeafGrowAndReturnKey() {
    auto leaf = DataLeafNode::CreateNewNode(blockStore, LAYOUT, Data(LAYOUT.maxBytesPerLeaf()));
    leaf->resize(5);
    return leaf->blockId();
  }

  unique_ref<BlockStore> _blockStore;
  BlockStore *blockStore;
  unique_ref<DataNodeStore> nodeStore;
  Data ZEROES;
  Data randomData;
  unique_ref<DataLeafNode> leaf;

private:
  DISALLOW_COPY_AND_ASSIGN(DataLeafNodeTest);
};

constexpr uint32_t DataLeafNodeTest::BLOCKSIZE_BYTES;
constexpr DataNodeLayout DataLeafNodeTest::LAYOUT;

class DataLeafNodeSizeTest: public DataLeafNodeTest, public WithParamInterface<unsigned int> {
public:
  BlockId CreateLeafResizeItAndReturnKey() {
    auto leaf = nodeStore->createNewLeafNode(Data(0));
    leaf->resize(GetParam());
    return leaf->blockId();
  }
};
INSTANTIATE_TEST_SUITE_P(DataLeafNodeSizeTest, DataLeafNodeSizeTest, Values(0, 1, 5, 16, 32, 512, DataNodeLayout(DataLeafNodeTest::BLOCKSIZE_BYTES).maxBytesPerLeaf()));

TEST_P(DataLeafNodeSizeTest, ResizeNode_ReadSizeImmediately) {
  leaf->resize(GetParam());
  EXPECT_EQ(GetParam(), leaf->numBytes());
}

TEST_P(DataLeafNodeSizeTest, ResizeNode_ReadSizeAfterLoading) {
  BlockId blockId = CreateLeafResizeItAndReturnKey();

  auto leaf = LoadLeafNode(blockId);
  EXPECT_EQ(GetParam(), leaf->numBytes());
}

TEST_F(DataLeafNodeTest, SpaceIsZeroFilledWhenGrowing) {
  leaf->resize(randomData.size());
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), loadData(*leaf).data(), randomData.size()));
}

TEST_F(DataLeafNodeTest, SpaceGetsZeroFilledWhenShrinkingAndRegrowing) {
  FillLeafBlockWithData();
  // resize it smaller and then back to original size
  uint32_t smaller_size = randomData.size() - 100;
  leaf->resize(smaller_size);
  leaf->resize(randomData.size());

  //Check that the space was filled with zeroes
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), static_cast<const uint8_t*>(loadData(*leaf).data())+smaller_size, 100));
}

TEST_F(DataLeafNodeTest, DataGetsZeroFilledWhenShrinking) {
  BlockId blockId = WriteDataToNewLeafBlockAndReturnKey();
  uint32_t smaller_size = randomData.size() - 100;
  {
    //At first, we expect there to be random data in the underlying data block
    auto block = blockStore->load(blockId).value();
    EXPECT_EQ(0, std::memcmp(randomData.dataOffset(smaller_size), static_cast<const uint8_t*>(block->data())+DataNodeLayout::HEADERSIZE_BYTES+smaller_size, 100));
  }

  //After shrinking, we expect there to be zeroes in the underlying data block
  ResizeLeaf(blockId, smaller_size);
  {
    auto block = blockStore->load(blockId).value();
    EXPECT_EQ(0, std::memcmp(ZEROES.data(), static_cast<const uint8_t*>(block->data())+DataNodeLayout::HEADERSIZE_BYTES+smaller_size, 100));
  }
}

TEST_F(DataLeafNodeTest, ShrinkingDoesntDestroyValidDataRegion) {
  FillLeafBlockWithData();
  uint32_t smaller_size = randomData.size() - 100;
  leaf->resize(smaller_size);

  //Check that the remaining data region is unchanged
  EXPECT_EQ(0, std::memcmp(randomData.data(), loadData(*leaf).data(), smaller_size));
}

TEST_F(DataLeafNodeTest, ConvertToInternalNode) {
  auto child = nodeStore->createNewLeafNode(Data(0));
  BlockId leaf_blockId = leaf->blockId();
  unique_ref<DataInnerNode> converted = DataNode::convertToNewInnerNode(std::move(leaf), LAYOUT, *child);

  EXPECT_EQ(1u, converted->numChildren());
  EXPECT_EQ(child->blockId(), converted->readChild(0).blockId());
  EXPECT_EQ(leaf_blockId, converted->blockId());
}

TEST_F(DataLeafNodeTest, ConvertToInternalNodeZeroesOutChildrenRegion) {
  BlockId blockId = CreateLeafWithDataConvertItToInnerNodeAndReturnKey();

  auto block = blockStore->load(blockId).value();
  EXPECT_EQ(0, std::memcmp(ZEROES.data(), static_cast<const uint8_t*>(block->data())+DataNodeLayout::HEADERSIZE_BYTES+sizeof(DataInnerNode::ChildEntry), nodeStore->layout().maxBytesPerLeaf()-sizeof(DataInnerNode::ChildEntry)));
}

TEST_F(DataLeafNodeTest, CopyingCreatesANewLeaf) {
  auto copied = CopyLeafNode(*leaf);
  EXPECT_NE(leaf->blockId(), copied->blockId());
}

TEST_F(DataLeafNodeTest, CopyEmptyLeaf) {
  auto copied = CopyLeafNode(*leaf);
  EXPECT_EQ(leaf->numBytes(), copied->numBytes());
}

TEST_F(DataLeafNodeTest, CopyDataLeaf) {
  FillLeafBlockWithData();
  auto copied = CopyLeafNode(*leaf);

  EXPECT_EQ(leaf->numBytes(), copied->numBytes());
  EXPECT_EQ(0, std::memcmp(loadData(*leaf).data(), loadData(*copied).data(), leaf->numBytes()));

  //Test that they have different data regions (changing the original one doesn't change the copy)
  uint8_t data = 0;
  leaf->write(&data, 0, 1);
  EXPECT_EQ(data, deserialize<uint8_t>(loadData(*leaf).data()));
  EXPECT_NE(data, deserialize<uint8_t>(loadData(*copied).data()));
}


struct DataRange {
  uint64_t leafsize;
  uint64_t offset;
  uint64_t count;
};

class DataLeafNodeDataTest: public DataLeafNodeTest, public WithParamInterface<DataRange> {
public:
  Data foregroundData;
  Data backgroundData;

  DataLeafNodeDataTest():
    foregroundData(DataFixture::generate(GetParam().count, 0)),
    backgroundData(DataFixture::generate(GetParam().leafsize, 1)) {
  }

  BlockId CreateLeafWriteToItAndReturnKey(const Data &to_write) {
    auto newleaf = nodeStore->createNewLeafNode(Data(0));

    newleaf->resize(GetParam().leafsize);
    newleaf->write(to_write.data(), GetParam().offset, GetParam().count);
    return newleaf->blockId();
  }

  void EXPECT_DATA_READS_AS(const Data &expected, const DataLeafNode &leaf, uint64_t offset, uint64_t count) {
    Data read(count);
    leaf.read(read.data(), offset, count);
    EXPECT_EQ(expected, read);
  }

  void EXPECT_DATA_READS_AS_OUTSIDE_OF(const Data &expected, const DataLeafNode &leaf, uint64_t start, uint64_t count) {
    Data begin(start);
    Data end(GetParam().leafsize - count - start);

    std::memcpy(begin.data(), expected.data(), start);
    std::memcpy(end.data(), expected.dataOffset(start+count), end.size());

    EXPECT_DATA_READS_AS(begin, leaf, 0, start);
    EXPECT_DATA_READS_AS(end, leaf, start + count, end.size());
  }

  void EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(const DataLeafNode &leaf, uint64_t start, uint64_t count) {
    Data ZEROES(GetParam().leafsize);
    ZEROES.FillWithZeroes();
    EXPECT_DATA_READS_AS_OUTSIDE_OF(ZEROES, leaf, start, count);
  }
};
INSTANTIATE_TEST_SUITE_P(DataLeafNodeDataTest, DataLeafNodeDataTest, Values(
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf(),     0,   DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()},     // full size leaf, access beginning to end
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf(),     100, DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-200}, // full size leaf, access middle to middle
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf(),     0,   DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100}, // full size leaf, access beginning to middle
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf(),     100, DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100}, // full size leaf, access middle to end
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100, 0,   DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100}, // non-full size leaf, access beginning to end
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100, 100, DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-300}, // non-full size leaf, access middle to middle
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100, 0,   DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-200}, // non-full size leaf, access beginning to middle
  DataRange{DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-100, 100, DataLeafNodeTest::LAYOUT.maxBytesPerLeaf()-200}  // non-full size leaf, access middle to end
));

TEST_P(DataLeafNodeDataTest, WriteAndReadImmediately) {
  leaf->resize(GetParam().leafsize);
  leaf->write(this->foregroundData.data(), GetParam().offset, GetParam().count);

  EXPECT_DATA_READS_AS(this->foregroundData, *leaf, GetParam().offset, GetParam().count);
  EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*leaf, GetParam().offset, GetParam().count);
}

TEST_P(DataLeafNodeDataTest, WriteAndReadAfterLoading) {
  BlockId blockId = CreateLeafWriteToItAndReturnKey(this->foregroundData);

  auto loaded_leaf = LoadLeafNode(blockId);
  EXPECT_DATA_READS_AS(this->foregroundData, *loaded_leaf, GetParam().offset, GetParam().count);
  EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*loaded_leaf, GetParam().offset, GetParam().count);
}

TEST_P(DataLeafNodeDataTest, OverwriteAndRead) {
  leaf->resize(GetParam().leafsize);
  leaf->write(this->backgroundData.data(), 0, GetParam().leafsize);
  leaf->write(this->foregroundData.data(), GetParam().offset, GetParam().count);
  EXPECT_DATA_READS_AS(this->foregroundData, *leaf, GetParam().offset, GetParam().count);
  EXPECT_DATA_READS_AS_OUTSIDE_OF(this->backgroundData, *leaf, GetParam().offset, GetParam().count);
}

}
