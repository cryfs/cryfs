#include <cpp-utils/data/DataFixture.h>
#include <gtest/gtest.h>

#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/tempfile/TempDir.h>

// TODO This should be ported to BlockStore2
/*
using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::TempFile;
using cpputils::TempDir;
using cpputils::unique_ref;

using namespace blockstore;
using namespace blockstore::ondisk;

namespace bf = boost::filesystem;

class OnDiskBlockFlushTest: public Test, public WithParamInterface<size_t> {
public:
  OnDiskBlockFlushTest()
  // Don't create the temp file yet (therefore pass false to the TempFile constructor)
  : dir(),
    key(BlockId::FromString("1491BB4932A389EE14BC7090AC772972")),
    file(dir.path() / blockId.ToString().substr(0,3) / blockId.ToString().substr(3), false),
    randomData(DataFixture::generate(GetParam())) {
  }
  TempDir dir;
  BlockId key;
  TempFile file;

  Data randomData;

  unique_ref<OnDiskBlock> CreateBlockAndLoadItFromDisk() {
    {
      OnDiskBlock::CreateOnDisk(dir.path(), blockId, randomData.copy()).value();
    }
    return OnDiskBlock::LoadFromDisk(dir.path(), blockId).value();
  }

  unique_ref<OnDiskBlock> CreateBlock() {
    return OnDiskBlock::CreateOnDisk(dir.path(), blockId, randomData.copy()).value();
  }

  void WriteDataToBlock(const unique_ref<OnDiskBlock> &block) {
    block->write(randomData.data(), 0, randomData.size());
  }

  void EXPECT_BLOCK_DATA_CORRECT(const unique_ref<OnDiskBlock> &block) {
    EXPECT_EQ(randomData.size(), block->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), block->data(), randomData.size()));
  }

  void EXPECT_STORED_FILE_DATA_CORRECT() {
    Data fileContent = Data::LoadFromFile(file.path()).value();
    Data fileContentWithoutHeader(fileContent.size() - OnDiskBlock::formatVersionHeaderSize());
    std::memcpy(fileContentWithoutHeader.data(), fileContent.dataOffset(OnDiskBlock::formatVersionHeaderSize()), fileContentWithoutHeader.size());
    EXPECT_EQ(randomData, fileContentWithoutHeader);
  }
};
INSTANTIATE_TEST_SUITE_P(OnDiskBlockFlushTest, OnDiskBlockFlushTest, Values((size_t)0, (size_t)1, (size_t)1024, (size_t)4096, (size_t)10*1024*1024));

// This test is also tested by OnDiskBlockStoreTest, but there the block is created using the BlockStore interface.
// Here, we create it using OnDiskBlock::CreateOnDisk()
TEST_P(OnDiskBlockFlushTest, AfterCreate_FlushingDoesntChangeBlock) {
  auto block =  CreateBlock();
  WriteDataToBlock(block);

  EXPECT_BLOCK_DATA_CORRECT(block);
}

// This test is also tested by OnDiskBlockStoreTest, but there the block is created using the BlockStore interface.
// Here, we create it using OnDiskBlock::CreateOnDisk() / OnDiskBlock::LoadFromDisk()
TEST_P(OnDiskBlockFlushTest, AfterLoad_FlushingDoesntChangeBlock) {
  auto block =  CreateBlockAndLoadItFromDisk();
  WriteDataToBlock(block);

  EXPECT_BLOCK_DATA_CORRECT(block);
}

TEST_P(OnDiskBlockFlushTest, AfterCreate_FlushingWritesCorrectData) {
  auto block = CreateBlock();
  WriteDataToBlock(block);

  EXPECT_STORED_FILE_DATA_CORRECT();
}

TEST_P(OnDiskBlockFlushTest, AfterLoad_FlushingWritesCorrectData) {
  auto block = CreateBlockAndLoadItFromDisk();
  WriteDataToBlock(block);

  EXPECT_STORED_FILE_DATA_CORRECT();
}

// This test is also tested by OnDiskBlockStoreTest, but there it can only checks block content by loading it again.
// Here, we check the content on disk.
TEST_P(OnDiskBlockFlushTest, AfterCreate_FlushesWhenDestructed) {
  {
    auto block = CreateBlock();
    WriteDataToBlock(block);
  }
  EXPECT_STORED_FILE_DATA_CORRECT();
}

// This test is also tested by OnDiskBlockStoreTest, but there it can only checks block content by loading it again.
// Here, we check the content on disk.
TEST_P(OnDiskBlockFlushTest, AfterLoad_FlushesWhenDestructed) {
  {
    auto block = CreateBlockAndLoadItFromDisk();
    WriteDataToBlock(block);
  }
  EXPECT_STORED_FILE_DATA_CORRECT();
}
*/
