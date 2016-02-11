#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_TESTUTILS_BLOCKSTORETEST_SIZE_H_
#define MESSMER_BLOCKSTORE_TEST_TESTUTILS_BLOCKSTORETEST_SIZE_H_

// This file is meant to be included by BlockStoreTest.h only

#include <cpp-utils/data/Data.h>
#include <cpp-utils/data/DataFixture.h>

class BlockStoreSizeParameterizedTest {
public:
  BlockStoreSizeParameterizedTest(cpputils::unique_ref<blockstore::BlockStore> blockStore_, size_t size_): blockStore(std::move(blockStore_)), size(size_) {}

  void TestCreatedBlockHasCorrectSize() {
    auto block = CreateBlock();
    EXPECT_EQ(size, block->size());
  }

  void TestLoadingUnchangedBlockHasCorrectSize() {
    blockstore::Key key = CreateBlock()->key();
    auto loaded_block = blockStore->load(key).value();
    EXPECT_EQ(size, loaded_block->size());
  }

  void TestCreatedBlockData() {
    cpputils::Data data = cpputils::DataFixture::generate(size);
    auto block = blockStore->create(data);
    EXPECT_EQ(0, std::memcmp(data.data(), block->data(), size));
  }

  void TestLoadingUnchangedBlockData() {
    cpputils::Data data = cpputils::DataFixture::generate(size);
    blockstore::Key key = blockStore->create(data)->key();
    auto loaded_block = blockStore->load(key).value();
    EXPECT_EQ(0, std::memcmp(data.data(), loaded_block->data(), size));
  }

  void TestLoadedBlockIsCorrect() {
    cpputils::Data randomData = cpputils::DataFixture::generate(size);
    auto loaded_block = StoreDataToBlockAndLoadIt(randomData);
    EXPECT_EQ(size, loaded_block->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_block->data(), size));
  }

  void TestLoadedBlockIsCorrectWhenLoadedDirectlyAfterFlushing() {
    cpputils::Data randomData = cpputils::DataFixture::generate(size);
    auto loaded_block = StoreDataToBlockAndLoadItDirectlyAfterFlushing(randomData);
    EXPECT_EQ(size, loaded_block->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_block->data(), size));
  }

  void TestAfterCreate_FlushingDoesntChangeBlock() {
    cpputils::Data randomData = cpputils::DataFixture::generate(size);
    auto block =  CreateBlock();
    WriteDataToBlock(block.get(), randomData);
    block->flush();

    EXPECT_BLOCK_DATA_CORRECT(*block, randomData);
  }

  void TestAfterLoad_FlushingDoesntChangeBlock() {
    cpputils::Data randomData = cpputils::DataFixture::generate(size);
    auto block =  CreateBlockAndLoadIt();
    WriteDataToBlock(block.get(), randomData);
    block->flush();

    EXPECT_BLOCK_DATA_CORRECT(*block, randomData);
  }

  void TestAfterCreate_FlushesWhenDestructed() {
    cpputils::Data randomData = cpputils::DataFixture::generate(size);
    blockstore::Key key = blockstore::Key::Null();
    {
      auto block = blockStore->create(cpputils::Data(size));
      key = block->key();
      WriteDataToBlock(block.get(), randomData);
    }
    auto loaded_block = blockStore->load(key).value();
    EXPECT_BLOCK_DATA_CORRECT(*loaded_block, randomData);
  }

  void TestAfterLoad_FlushesWhenDestructed() {
    cpputils::Data randomData = cpputils::DataFixture::generate(size);
    blockstore::Key key = blockstore::Key::Null();
    {
      key = CreateBlock()->key();
      auto block = blockStore->load(key).value();
      WriteDataToBlock(block.get(), randomData);
    }
    auto loaded_block = blockStore->load(key).value();
    EXPECT_BLOCK_DATA_CORRECT(*loaded_block, randomData);
  }

  void TestLoadNonExistingBlock() {
    EXPECT_EQ(boost::none, blockStore->load(key));
  }

private:
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  cpputils::unique_ref<blockstore::BlockStore> blockStore;
  size_t size;

  cpputils::Data ZEROES(size_t size) {
    cpputils::Data ZEROES(size);
    ZEROES.FillWithZeroes();
    return ZEROES;
  }

  cpputils::unique_ref<blockstore::Block> StoreDataToBlockAndLoadIt(const cpputils::Data &data) {
    blockstore::Key key = StoreDataToBlockAndGetKey(data);
    return blockStore->load(key).value();
  }

  blockstore::Key StoreDataToBlockAndGetKey(const cpputils::Data &data) {
    return blockStore->create(data)->key();
  }

  cpputils::unique_ref<blockstore::Block> StoreDataToBlockAndLoadItDirectlyAfterFlushing(const cpputils::Data &data) {
    auto block = blockStore->create(data);
    block->flush();
    return blockStore->load(block->key()).value();
  }

  cpputils::unique_ref<blockstore::Block> CreateBlockAndLoadIt() {
    blockstore::Key key = CreateBlock()->key();
    return blockStore->load(key).value();
  }

  cpputils::unique_ref<blockstore::Block> CreateBlock() {
    return blockStore->create(cpputils::Data(size));
  }

  void WriteDataToBlock(blockstore::Block *block, const cpputils::Data &randomData) {
    block->write(randomData.data(), 0, randomData.size());
  }

  void EXPECT_BLOCK_DATA_CORRECT(const blockstore::Block &block, const cpputils::Data &randomData) {
    EXPECT_EQ(randomData.size(), block.size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), block.data(), randomData.size()));
  }
};

constexpr std::initializer_list<size_t> SIZES = {0, 1, 1024, 4096, 10*1024*1024};
#define TYPED_TEST_P_FOR_ALL_SIZES(TestName)                                    \
  TYPED_TEST_P(BlockStoreTest, TestName) {                                      \
    for (auto size: SIZES) {                                                    \
      BlockStoreSizeParameterizedTest(this->fixture.createBlockStore(), size)   \
        .Test##TestName();                                                      \
    }                                                                           \
  }                                                                             \

TYPED_TEST_P_FOR_ALL_SIZES(CreatedBlockHasCorrectSize);
TYPED_TEST_P_FOR_ALL_SIZES(LoadingUnchangedBlockHasCorrectSize);
TYPED_TEST_P_FOR_ALL_SIZES(CreatedBlockData);
TYPED_TEST_P_FOR_ALL_SIZES(LoadingUnchangedBlockData);
TYPED_TEST_P_FOR_ALL_SIZES(LoadedBlockIsCorrect);
//TYPED_TEST_P_FOR_ALL_SIZES(LoadedBlockIsCorrectWhenLoadedDirectlyAfterFlushing);
TYPED_TEST_P_FOR_ALL_SIZES(AfterCreate_FlushingDoesntChangeBlock);
TYPED_TEST_P_FOR_ALL_SIZES(AfterLoad_FlushingDoesntChangeBlock);
TYPED_TEST_P_FOR_ALL_SIZES(AfterCreate_FlushesWhenDestructed);
TYPED_TEST_P_FOR_ALL_SIZES(AfterLoad_FlushesWhenDestructed);
TYPED_TEST_P_FOR_ALL_SIZES(LoadNonExistingBlock);

#endif
