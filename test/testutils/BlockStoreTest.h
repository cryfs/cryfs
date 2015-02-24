#pragma once
#ifndef TEST_BLOCKSTORE_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_
#define TEST_BLOCKSTORE_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_

#include "google/gtest/gtest.h"

#include "DataBlockFixture.h"

#include "../../interface/BlockStore.h"

class BlockStoreTestFixture {
public:
  virtual std::unique_ptr<blockstore::BlockStore> createBlockStore() = 0;
};

template<class ConcreteBlockStoreTestFixture>
class BlockStoreTest: public ::testing::Test {
public:
  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<BlockStoreTestFixture, ConcreteBlockStoreTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) BlockStoreTest must inherit from BlockStoreTestFixture"
  );

  const std::vector<size_t> SIZES = {0, 1, 1024, 4096, 10*1024*1024};

  ConcreteBlockStoreTestFixture fixture;
};

template<class ConcreateBlockStoreTestFixture>
class BlockStoreSizeParameterizedTest {
public:
  BlockStoreSizeParameterizedTest(ConcreateBlockStoreTestFixture &fixture, size_t size_): blockStore(fixture.createBlockStore()), size(size_) {}

  void TestCreatedBlockHasCorrectSize() {
    auto block = blockStore->create(size);
    EXPECT_EQ(size, block->size());
  }

  void TestLoadingUnchangedBlockHasCorrectSize() {
    auto block = blockStore->create(size);
    auto loaded_block = blockStore->load(block->key());
    EXPECT_EQ(size, loaded_block->size());
  }

  void TestCreatedBlockIsZeroedOut() {
    auto block = blockStore->create(size);
    EXPECT_EQ(0, std::memcmp(ZEROES(size).data(), block->data(), size));
  }

  void TestLoadingUnchangedBlockIsZeroedOut() {
    auto block = blockStore->create(size);
    auto loaded_block = blockStore->load(block->key());
    EXPECT_EQ(0, std::memcmp(ZEROES(size).data(), loaded_block->data(), size));
  }

  void TestLoadedBlockIsCorrect() {
    DataBlockFixture randomData(size);
    auto loaded_block = StoreDataToBlockAndLoadIt(randomData);
    EXPECT_EQ(size, loaded_block->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_block->data(), size));
  }

  void TestLoadedBlockIsCorrectWhenLoadedDirectlyAfterFlushing() {
    DataBlockFixture randomData(size);
    auto loaded_block = StoreDataToBlockAndLoadItDirectlyAfterFlushing(randomData);
    EXPECT_EQ(size, loaded_block->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_block->data(), size));
  }

  void TestAfterCreate_FlushingDoesntChangeBlock() {
    DataBlockFixture randomData(size);
    auto block =  CreateBlock();
    WriteDataToBlock(block.get(), randomData);
    block->flush();

    EXPECT_BLOCK_DATA_CORRECT(*block, randomData);
  }

  void TestAfterLoad_FlushingDoesntChangeBlock() {
    DataBlockFixture randomData(size);
    auto block =  CreateBlockAndLoadIt();
    WriteDataToBlock(block.get(), randomData);
    block->flush();

    EXPECT_BLOCK_DATA_CORRECT(*block, randomData);
  }

  void TestAfterCreate_FlushesWhenDestructed() {
    DataBlockFixture randomData(size);
    blockstore::Key key = key;
    {
      auto block = blockStore->create(size);
      key = block->key();
      WriteDataToBlock(block.get(), randomData);
    }
    auto loaded_block = blockStore->load(key);
    EXPECT_BLOCK_DATA_CORRECT(*loaded_block, randomData);
  }

  void TestAfterLoad_FlushesWhenDestructed() {
    DataBlockFixture randomData(size);
    blockstore::Key key = key;
    {
      key = blockStore->create(size)->key();
      auto block = blockStore->load(key);
      WriteDataToBlock(block.get(), randomData);
    }
    auto loaded_block = blockStore->load(key);
    EXPECT_BLOCK_DATA_CORRECT(*loaded_block, randomData);
  }

  void TestLoadNonExistingBlock() {
    EXPECT_FALSE(
        (bool)blockStore->load(key)
    );
  }

  void TestLoadNonExistingBlockWithEmptyKey() {
    EXPECT_FALSE(
        (bool)blockStore->load(key)
    );
  }

private:
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  std::unique_ptr<blockstore::BlockStore> blockStore;
  size_t size;

  blockstore::Data ZEROES(size_t size) {
    blockstore::Data ZEROES(size);
    ZEROES.FillWithZeroes();
    return ZEROES;
  }

  std::unique_ptr<blockstore::Block> StoreDataToBlockAndLoadIt(const DataBlockFixture &data) {
    blockstore::Key key = StoreDataToBlockAndGetKey(data);
    return blockStore->load(key);
  }

  blockstore::Key StoreDataToBlockAndGetKey(const DataBlockFixture &data) {
    auto block = blockStore->create(data.size());
    std::memcpy(block->data(), data.data(), data.size());
    return block->key();
  }

  std::unique_ptr<blockstore::Block> StoreDataToBlockAndLoadItDirectlyAfterFlushing(const DataBlockFixture &data) {
    auto block = blockStore->create(data.size());
    std::memcpy(block->data(), data.data(), data.size());
    block->flush();
    return blockStore->load(block->key());
  }

  std::unique_ptr<blockstore::Block> CreateBlockAndLoadIt() {
    blockstore::Key key = blockStore->create(size)->key();
    return blockStore->load(key);
  }

  std::unique_ptr<blockstore::Block> CreateBlock() {
    return blockStore->create(size);
  }

  void WriteDataToBlock(blockstore::Block *block, const DataBlockFixture &randomData) {
    std::memcpy(block->data(), randomData.data(), randomData.size());
  }

  void EXPECT_BLOCK_DATA_CORRECT(const blockstore::Block &block, const DataBlockFixture &randomData) {
    EXPECT_EQ(randomData.size(), block.size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), block.data(), randomData.size()));
  }
};

TYPED_TEST_CASE_P(BlockStoreTest);

#define TYPED_TEST_P_FOR_ALL_SIZES(TestName)                           \
  TYPED_TEST_P(BlockStoreTest, TestName) {                             \
    for (auto size: this->SIZES) {                                     \
      BlockStoreSizeParameterizedTest<TypeParam>(this->fixture, size)  \
        .Test##TestName();                                             \
    }                                                                  \
  }                                                                    \


TYPED_TEST_P_FOR_ALL_SIZES(CreatedBlockHasCorrectSize);
TYPED_TEST_P_FOR_ALL_SIZES(LoadingUnchangedBlockHasCorrectSize);
TYPED_TEST_P_FOR_ALL_SIZES(CreatedBlockIsZeroedOut);
TYPED_TEST_P_FOR_ALL_SIZES(LoadingUnchangedBlockIsZeroedOut);
TYPED_TEST_P_FOR_ALL_SIZES(LoadedBlockIsCorrect);
TYPED_TEST_P_FOR_ALL_SIZES(LoadedBlockIsCorrectWhenLoadedDirectlyAfterFlushing);
TYPED_TEST_P_FOR_ALL_SIZES(AfterCreate_FlushingDoesntChangeBlock);
TYPED_TEST_P_FOR_ALL_SIZES(AfterLoad_FlushingDoesntChangeBlock);
TYPED_TEST_P_FOR_ALL_SIZES(AfterCreate_FlushesWhenDestructed);
TYPED_TEST_P_FOR_ALL_SIZES(AfterLoad_FlushesWhenDestructed);
TYPED_TEST_P_FOR_ALL_SIZES(LoadNonExistingBlock);
TYPED_TEST_P_FOR_ALL_SIZES(LoadNonExistingBlockWithEmptyKey);

TYPED_TEST_P(BlockStoreTest, TwoCreatedBlocksHaveDifferentKeys) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(1024);
  auto block2 = blockStore->create(1024);
  EXPECT_NE(block1->key(), block2->key());
}

TYPED_TEST_P(BlockStoreTest, BlockIsNotLoadableAfterDeleting) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockkey = blockStore->create(1024)->key();
  auto block = blockStore->load(blockkey);
  EXPECT_NE(nullptr, block.get());
  blockStore->remove(std::move(block));
  EXPECT_EQ(nullptr, blockStore->load(blockkey).get());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectOnEmptyBlockstore) {
  auto blockStore = this->fixture.createBlockStore();
  EXPECT_EQ(0, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingOneBlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(1);
  EXPECT_EQ(1, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingTheLastBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(1);
  blockStore->remove(std::move(block));
  EXPECT_EQ(0, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocks) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(1);
  blockStore->create(0);
  EXPECT_EQ(2, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingABlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(1);
  blockStore->create(1);
  blockStore->remove(std::move(block));
  EXPECT_EQ(1, blockStore->numBlocks());
}

REGISTER_TYPED_TEST_CASE_P(BlockStoreTest,
    CreatedBlockHasCorrectSize,
    LoadingUnchangedBlockHasCorrectSize,
    CreatedBlockIsZeroedOut,
    LoadingUnchangedBlockIsZeroedOut,
    LoadedBlockIsCorrect,
    LoadedBlockIsCorrectWhenLoadedDirectlyAfterFlushing,
    AfterCreate_FlushingDoesntChangeBlock,
    AfterLoad_FlushingDoesntChangeBlock,
    AfterCreate_FlushesWhenDestructed,
    AfterLoad_FlushesWhenDestructed,
    LoadNonExistingBlock,
    LoadNonExistingBlockWithEmptyKey,
    TwoCreatedBlocksHaveDifferentKeys,
    BlockIsNotLoadableAfterDeleting,
    NumBlocksIsCorrectOnEmptyBlockstore,
    NumBlocksIsCorrectAfterAddingOneBlock,
    NumBlocksIsCorrectAfterRemovingTheLastBlock,
    NumBlocksIsCorrectAfterAddingTwoBlocks,
    NumBlocksIsCorrectAfterRemovingABlock
);


#endif
