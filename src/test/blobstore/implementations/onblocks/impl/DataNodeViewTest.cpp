#include <blobstore/implementations/onblocks/datanodestore/DataNodeView.h>
#include <gtest/gtest.h>

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlock.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include "test/testutils/DataBlockFixture.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;
using std::unique_ptr;
using std::make_unique;
using std::string;

using blockstore::BlockStore;
using blockstore::testfake::FakeBlockStore;
using namespace blobstore;
using namespace blobstore::onblocks;

class DataNodeViewTest: public Test {
public:
  //TODO Don't use InMemoryBlockStore for test cases, because it ignores flushing.
  //     And if you for example write more than the actual block size, it still will keep that data for you,
  //     because the next block load will just give you the same data region (and the overflow data will most
  //     likely still be intact).
  //     So better write a FakeBlockStore class for test cases.
  unique_ptr<BlockStore> blockStore = make_unique<FakeBlockStore>();
};

class DataNodeViewDepthTest: public DataNodeViewTest, public WithParamInterface<uint8_t> {
};
INSTANTIATE_TEST_CASE_P(DataNodeViewDepthTest, DataNodeViewDepthTest, Values(0, 1, 3, 10, 100));

TEST_P(DataNodeViewDepthTest, DepthIsStored) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  {
    DataNodeView view(std::move(block.block));
    *view.Depth() = GetParam();
  }
  DataNodeView view(blockStore->load(block.key));
  EXPECT_EQ(GetParam(), *view.Depth());
}

class DataNodeViewSizeTest: public DataNodeViewTest, public WithParamInterface<uint32_t> {
};
INSTANTIATE_TEST_CASE_P(DataNodeViewSizeTest, DataNodeViewSizeTest, Values(0, 50, 64, 1024, 1024*1024*1024));

TEST_P(DataNodeViewSizeTest, SizeIsStored) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  {
    DataNodeView view(std::move(block.block));
    *view.Size() = GetParam();
  }
  DataNodeView view(blockStore->load(block.key));
  EXPECT_EQ(GetParam(), *view.Size());
}

TEST_F(DataNodeViewTest, DataIsStored) {
  DataBlockFixture randomData(DataNodeView::DATASIZE_BYTES);
  auto block = blockStore->create(DataNodeView::BLOCKSIZE_BYTES);
  {
    DataNodeView view(std::move(block.block));
    std::memcpy(view.DataBegin<uint8_t>(), randomData.data(), randomData.size());
  }
  DataNodeView view(blockStore->load(block.key));
  EXPECT_EQ(0, std::memcmp(view.DataBegin<uint8_t>(), randomData.data(), randomData.size()));
}

TEST_F(DataNodeViewTest, HeaderAndBodyDontOverlap) {
  DataBlockFixture randomData(DataNodeView::DATASIZE_BYTES);
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  {
    DataNodeView view(std::move(block.block));
    *view.Depth() = 3;
    *view.Size() = 1000000000u;
    std::memcpy(view.DataBegin<uint8_t>(), randomData.data(), DataNodeView::DATASIZE_BYTES);
  }
  DataNodeView view(blockStore->load(block.key));
  EXPECT_EQ(3, *view.Depth());
  EXPECT_EQ(1000000000u, *view.Size());
  EXPECT_EQ(0, std::memcmp(view.DataBegin<uint8_t>(), randomData.data(), DataNodeView::DATASIZE_BYTES));
}

TEST_F(DataNodeViewTest, DataBeginWorksWithOneByteEntries) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  uint8_t *blockBegin = (uint8_t*)block.block->data();
  DataNodeView view(std::move(block.block));

  EXPECT_EQ(blockBegin+view.HEADERSIZE_BYTES, view.DataBegin<uint8_t>());
}

TEST_F(DataNodeViewTest, DataBeginWorksWithEightByteEntries) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  uint8_t *blockBegin = (uint8_t*)block.block->data();
  DataNodeView view(std::move(block.block));

  EXPECT_EQ(blockBegin+view.HEADERSIZE_BYTES, (uint8_t*)view.DataBegin<uint64_t>());
}

TEST_F(DataNodeViewTest, DataEndWorksWithOneByteEntries) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  uint8_t *blockBegin = (uint8_t*)block.block->data();
  DataNodeView view(std::move(block.block));

  EXPECT_EQ(blockBegin+view.BLOCKSIZE_BYTES, view.DataEnd<uint8_t>());
}

TEST_F(DataNodeViewTest, DataEndWorksWithEightByteEntries) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  uint8_t *blockBegin = (uint8_t*)block.block->data();
  DataNodeView view(std::move(block.block));

  EXPECT_EQ(blockBegin+view.BLOCKSIZE_BYTES, (uint8_t*)view.DataEnd<uint64_t>());
}

struct SizedDataEntry {
  uint8_t data[6];
};
BOOST_STATIC_ASSERT_MSG(DataNodeView::DATASIZE_BYTES % sizeof(SizedDataEntry) != 0,
  "This test case only makes sense, if the data entries don't fill up the whole space. "
  "There should be some space left at the end that is not used, because it isn't enough space for a full entry. "
  "If this static assertion fails, please use a different size for SizedDataEntry.");

TEST_F(DataNodeViewTest, DataBeginWorksWithStructEntries) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  uint8_t *blockBegin = (uint8_t*)block.block->data();
  DataNodeView view(std::move(block.block));

  EXPECT_EQ(blockBegin+view.HEADERSIZE_BYTES, (uint8_t*)view.DataBegin<SizedDataEntry>());
}

TEST_F(DataNodeViewTest, DataEndWorksWithStructByteEntries) {
  auto block = blockStore->create(BlobStoreOnBlocks::BLOCKSIZE);
  uint8_t *blockBegin = (uint8_t*)block.block->data();
  DataNodeView view(std::move(block.block));

  unsigned int numFittingEntries = view.DATASIZE_BYTES / sizeof(SizedDataEntry);

  uint8_t *dataEnd = (uint8_t*)view.DataEnd<SizedDataEntry>();
  EXPECT_EQ(blockBegin+view.HEADERSIZE_BYTES + numFittingEntries * sizeof(SizedDataEntry), dataEnd);
  EXPECT_LT(dataEnd, blockBegin + view.BLOCKSIZE_BYTES);
}
