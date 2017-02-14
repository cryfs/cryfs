#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_

#include <gtest/gtest.h>
#include <cpp-utils/data/DataFixture.h>
#include "blockstore/interface/BlockStore.h"

class BlockStoreTestFixture {
public:
  virtual ~BlockStoreTestFixture() {}
  virtual cpputils::unique_ref<blockstore::BlockStore> createBlockStore() = 0;
};

template<class ConcreteBlockStoreTestFixture>
class BlockStoreTest: public ::testing::Test {
public:
  BlockStoreTest() :fixture() {}

  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<BlockStoreTestFixture, ConcreteBlockStoreTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) BlockStoreTest must inherit from BlockStoreTestFixture"
  );

  ConcreteBlockStoreTestFixture fixture;

  void TestBlockIsUsable(cpputils::unique_ref<blockstore::Block> block, blockstore::BlockStore *blockStore) {
    // Write full block space and check it was correctly written
    cpputils::Data fixture = cpputils::DataFixture::generate(block->size());
    block->write(fixture.data(), 0, fixture.size());
    EXPECT_EQ(0, std::memcmp(fixture.data(), block->data(), fixture.size()));

    // Store and reload block and check data is still correct
    auto key = block->key();
    cpputils::destruct(std::move(block));
    block = blockStore->load(key).value();
    EXPECT_EQ(0, std::memcmp(fixture.data(), block->data(), fixture.size()));
  }
};

TYPED_TEST_CASE_P(BlockStoreTest);

TYPED_TEST_P(BlockStoreTest, TwoCreatedBlocksHaveDifferentKeys) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1024));
  auto block2 = blockStore->create(cpputils::Data(1024));
  EXPECT_NE(block1->key(), block2->key());
}

TYPED_TEST_P(BlockStoreTest, BlockIsNotLoadableAfterDeleting) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockkey = blockStore->create(cpputils::Data(1024))->key();
  auto block = blockStore->load(blockkey);
  EXPECT_NE(boost::none, block);
  blockStore->remove(std::move(*block));
  EXPECT_EQ(boost::none, blockStore->load(blockkey));
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectOnEmptyBlockstore) {
  auto blockStore = this->fixture.createBlockStore();
  EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingOneBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(1));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingOneBlock_AfterClosingBlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(cpputils::Data(1));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingTheLastBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(1));
  blockStore->remove(std::move(block));
  EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1));
  auto block2 = blockStore->create(cpputils::Data(0));
  EXPECT_EQ(2u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingFirstBlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(cpputils::Data(1));
  auto block2 = blockStore->create(cpputils::Data(0));
  EXPECT_EQ(2u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingSecondBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1));
  blockStore->create(cpputils::Data(0));
  EXPECT_EQ(2u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingBothBlocks) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(cpputils::Data(1));
  blockStore->create(cpputils::Data(0));
  EXPECT_EQ(2u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingABlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(1));
  blockStore->create(cpputils::Data(1));
  blockStore->remove(std::move(block));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, CanRemoveModifiedBlock) {
    auto blockStore = this->fixture.createBlockStore();
    auto block = blockStore->create(cpputils::Data(5));
    block->write("data", 0, 4);
    blockStore->remove(std::move(block));
    EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, Resize_Larger_FromZero) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(0));
  block->resize(10);
  EXPECT_EQ(10u, block->size());
}

TYPED_TEST_P(BlockStoreTest, Resize_Larger_FromZero_BlockIsStillUsable) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(0));
  block->resize(10);
  this->TestBlockIsUsable(std::move(block), blockStore.get());
}

TYPED_TEST_P(BlockStoreTest, Resize_Larger) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(10));
  block->resize(20);
  EXPECT_EQ(20u, block->size());
}

TYPED_TEST_P(BlockStoreTest, Resize_Larger_BlockIsStillUsable) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(10));
  block->resize(20);
  this->TestBlockIsUsable(std::move(block), blockStore.get());
}

TYPED_TEST_P(BlockStoreTest, Resize_Smaller) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(10));
  block->resize(5);
  EXPECT_EQ(5u, block->size());
}

TYPED_TEST_P(BlockStoreTest, Resize_Smaller_BlockIsStillUsable) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(10));
  block->resize(5);
  this->TestBlockIsUsable(std::move(block), blockStore.get());
}

TYPED_TEST_P(BlockStoreTest, Resize_Smaller_ToZero) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(10));
  block->resize(0);
  EXPECT_EQ(0u, block->size());
}

TYPED_TEST_P(BlockStoreTest, Resize_Smaller_ToZero_BlockIsStillUsable) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(10));
  block->resize(0);
  this->TestBlockIsUsable(std::move(block), blockStore.get());
}

#include "BlockStoreTest_Size.h"
#include "BlockStoreTest_Data.h"


REGISTER_TYPED_TEST_CASE_P(BlockStoreTest,
    CreatedBlockHasCorrectSize,
    LoadingUnchangedBlockHasCorrectSize,
    CreatedBlockData,
    LoadingUnchangedBlockData,
    LoadedBlockIsCorrect,
//    LoadedBlockIsCorrectWhenLoadedDirectlyAfterFlushing,
    AfterCreate_FlushingDoesntChangeBlock,
    AfterLoad_FlushingDoesntChangeBlock,
    AfterCreate_FlushesWhenDestructed,
    AfterLoad_FlushesWhenDestructed,
    LoadNonExistingBlock,
    TwoCreatedBlocksHaveDifferentKeys,
    BlockIsNotLoadableAfterDeleting,
    NumBlocksIsCorrectOnEmptyBlockstore,
    NumBlocksIsCorrectAfterAddingOneBlock,
    NumBlocksIsCorrectAfterAddingOneBlock_AfterClosingBlock,
    NumBlocksIsCorrectAfterRemovingTheLastBlock,
    NumBlocksIsCorrectAfterAddingTwoBlocks,
    NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingFirstBlock,
    NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingSecondBlock,
    NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingBothBlocks,
    NumBlocksIsCorrectAfterRemovingABlock,
    WriteAndReadImmediately,
    WriteAndReadAfterLoading,
    OverwriteAndRead,
    CanRemoveModifiedBlock,
    Resize_Larger_FromZero,
    Resize_Larger_FromZero_BlockIsStillUsable,
    Resize_Larger,
    Resize_Larger_BlockIsStillUsable,
    Resize_Smaller,
    Resize_Smaller_BlockIsStillUsable,
    Resize_Smaller_ToZero,
    Resize_Smaller_ToZero_BlockIsStillUsable
);


#endif
