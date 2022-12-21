#include "blobstore/implementations/onblocks/datanodestore/DataNodeView.h"
#include <gtest/gtest.h>

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <blockstore/implementations/testfake/FakeBlock.h>
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"
#include <cpp-utils/data/DataFixture.h>

using std::string;
using ::testing::Test;
using ::testing::Values;
using ::testing::WithParamInterface;

using blockstore::BlockStore;
using blockstore::testfake::FakeBlockStore;
using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

class DataNodeViewTest : public Test
{
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;
  static constexpr uint32_t DATASIZE_BYTES = DataNodeLayout(DataNodeViewTest::BLOCKSIZE_BYTES).datasizeBytes();

  unique_ref<BlockStore> blockStore = make_unique_ref<FakeBlockStore>();
};

class DataNodeViewDepthTest : public DataNodeViewTest, public WithParamInterface<uint8_t>
{
};
INSTANTIATE_TEST_SUITE_P(DataNodeViewDepthTest, DataNodeViewDepthTest, Values(0, 1, 3, 10, 100));

TEST_P(DataNodeViewDepthTest, DepthIsStored)
{
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  auto blockId = block->blockId();
  {
    DataNodeView view(std::move(block));
    view.setDepth(GetParam());
  }
  DataNodeView view(blockStore->load(blockId).value());
  EXPECT_EQ(GetParam(), view.Depth());
}

class DataNodeViewSizeTest : public DataNodeViewTest, public WithParamInterface<uint32_t>
{
};
INSTANTIATE_TEST_SUITE_P(DataNodeViewSizeTest, DataNodeViewSizeTest, Values(0, 50, 64, 1024, 1024 * 1024 * 1024));

TEST_P(DataNodeViewSizeTest, SizeIsStored)
{
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  auto blockId = block->blockId();
  {
    DataNodeView view(std::move(block));
    view.setSize(GetParam());
  }
  DataNodeView view(blockStore->load(blockId).value());
  EXPECT_EQ(GetParam(), view.Size());
}

TEST_F(DataNodeViewTest, DataIsStored)
{
  Data randomData = DataFixture::generate(DATASIZE_BYTES);
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  auto blockId = block->blockId();
  {
    DataNodeView view(std::move(block));
    view.write(randomData.data(), 0, randomData.size());
  }
  DataNodeView view(blockStore->load(blockId).value());
  EXPECT_EQ(0, std::memcmp(view.data(), randomData.data(), randomData.size()));
}

TEST_F(DataNodeViewTest, HeaderAndBodyDontOverlap)
{
  Data randomData = DataFixture::generate(DATASIZE_BYTES);
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  auto blockId = block->blockId();
  {
    DataNodeView view(std::move(block));
    view.setDepth(3);
    view.setSize(1000000000u);
    view.write(randomData.data(), 0, DATASIZE_BYTES);
  }
  DataNodeView view(blockStore->load(blockId).value());
  EXPECT_EQ(3, view.Depth());
  EXPECT_EQ(1000000000u, view.Size());
  EXPECT_EQ(0, std::memcmp(view.data(), randomData.data(), DATASIZE_BYTES));
}

TEST_F(DataNodeViewTest, Data)
{
  auto block = blockStore->create(Data(BLOCKSIZE_BYTES));
  const uint8_t *blockBegin = static_cast<const uint8_t *>(block->data());
  DataNodeView view(std::move(block));

  EXPECT_EQ(blockBegin + DataNodeLayout::HEADERSIZE_BYTES, static_cast<const uint8_t *>(view.data()));
}

// TODO Test that header fields (and data) are also stored over reloads
