#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

#include "blockstore/interface/BlockStore.h"

class MockForEachBlockCallback final {
public:
    std::function<void (const blockstore::BlockId &)> callback() {
      return [this] (const blockstore::BlockId &blockId) {
          called_with.push_back(blockId);
      };
    }

    std::vector<blockstore::BlockId> called_with;
};

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
    auto blockId = block->blockId();
    cpputils::destruct(std::move(block));
    block = blockStore->load(blockId).value();
    EXPECT_EQ(0, std::memcmp(fixture.data(), block->data(), fixture.size()));
  }

  template<class Entry>
  void EXPECT_UNORDERED_EQ(const std::vector<Entry> &expected, std::vector<Entry> actual) {
    EXPECT_EQ(expected.size(), actual.size());
    for (const Entry &expectedEntry : expected) {
      removeOne(&actual, expectedEntry);
    }
  }

  template<class Entry>
  void removeOne(std::vector<Entry> *entries, const Entry &toRemove) {
    auto found = std::find(entries->begin(), entries->end(), toRemove);
    if (found != entries->end()) {
      entries->erase(found);
      return;
    }
    EXPECT_TRUE(false);
  }
};

TYPED_TEST_SUITE_P(BlockStoreTest);

TYPED_TEST_P(BlockStoreTest, TwoCreatedBlocksHaveDifferentBlockIds) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1024));
  auto block2 = blockStore->create(cpputils::Data(1024));
  EXPECT_NE(block1->blockId(), block2->blockId());
}

TYPED_TEST_P(BlockStoreTest, BlockIsNotLoadableAfterDeleting_DeleteByBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId = blockStore->create(cpputils::Data(1024))->blockId();
  auto block = blockStore->load(blockId);
  EXPECT_NE(boost::none, block);
  blockStore->remove(std::move(*block));
  EXPECT_EQ(boost::none, blockStore->load(blockId));
}

TYPED_TEST_P(BlockStoreTest, BlockIsNotLoadableAfterDeleting_DeleteByBlockId) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId = blockStore->create(cpputils::Data(1024))->blockId();
  blockStore->remove(blockId);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
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

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingTheLastBlock_DeleteByBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(1));
  blockStore->remove(std::move(block));
  EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingTheLastBlock_DeleteByBlockId) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId = blockStore->create(cpputils::Data(1))->blockId();
  blockStore->remove(blockId);
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

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingABlock_DeleteByBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(1));
  blockStore->create(cpputils::Data(1));
  blockStore->remove(std::move(block));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, NumBlocksIsCorrectAfterRemovingABlock_DeleteByBlockId) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId = blockStore->create(cpputils::Data(1))->blockId();
  blockStore->create(cpputils::Data(1));
  blockStore->remove(blockId);
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, CanRemoveModifiedBlock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(5));
  block->write("data", 0, 4);
  blockStore->remove(std::move(block));
  EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStoreTest, ForEachBlock_zeroblocks) {
  auto blockStore = this->fixture.createBlockStore();
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStoreTest, ForEachBlock_oneblock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(cpputils::Data(1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({block->blockId()}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStoreTest, ForEachBlock_twoblocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1));
  auto block2 = blockStore->create(cpputils::Data(1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({block1->blockId(), block2->blockId()}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStoreTest, ForEachBlock_threeblocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1));
  auto block2 = blockStore->create(cpputils::Data(1));
  auto block3 = blockStore->create(cpputils::Data(1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({block1->blockId(), block2->blockId(), block3->blockId()}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStoreTest, ForEachBlock_doesntListRemovedBlocks_oneblock) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1));
  blockStore->remove(std::move(block1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStoreTest, ForEachBlock_doesntListRemovedBlocks_twoblocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto block1 = blockStore->create(cpputils::Data(1));
  auto block2 = blockStore->create(cpputils::Data(1));
  blockStore->remove(std::move(block1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({block2->blockId()}, mockForEachBlockCallback.called_with);
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
/*
TYPED_TEST_P(BlockStoreTest, TryCreateTwoBlocksWithSameBlockIdAndSameSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto block = blockStore->tryCreate(blockId, cpputils::Data(1024));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(blockId, cpputils::Data(1024));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreTest, TryCreateTwoBlocksWithSameBlockIdAndDifferentSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto block = blockStore->tryCreate(blockId, cpputils::Data(1024));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(blockId, cpputils::Data(4096));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreTest, TryCreateTwoBlocksWithSameBlockIdAndFirstNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto block = blockStore->tryCreate(blockId, cpputils::Data(0));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(blockId, cpputils::Data(1024));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreTest, TryCreateTwoBlocksWithSameBlockIdAndSecondNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto block = blockStore->tryCreate(blockId, cpputils::Data(1024));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(blockId, cpputils::Data(0));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreTest, TryCreateTwoBlocksWithSameBlockIdAndBothNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto block = blockStore->tryCreate(blockId, cpputils::Data(0));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(blockId, cpputils::Data(0));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}*/

#include "BlockStoreTest_Size.h"
#include "BlockStoreTest_Data.h"


REGISTER_TYPED_TEST_SUITE_P(BlockStoreTest,
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
    TwoCreatedBlocksHaveDifferentBlockIds,
    BlockIsNotLoadableAfterDeleting_DeleteByBlock,
    BlockIsNotLoadableAfterDeleting_DeleteByBlockId,
    NumBlocksIsCorrectOnEmptyBlockstore,
    NumBlocksIsCorrectAfterAddingOneBlock,
    NumBlocksIsCorrectAfterAddingOneBlock_AfterClosingBlock,
    NumBlocksIsCorrectAfterRemovingTheLastBlock_DeleteByBlock,
    NumBlocksIsCorrectAfterRemovingTheLastBlock_DeleteByBlockId,
    NumBlocksIsCorrectAfterAddingTwoBlocks,
    NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingFirstBlock,
    NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingSecondBlock,
    NumBlocksIsCorrectAfterAddingTwoBlocks_AfterClosingBothBlocks,
    NumBlocksIsCorrectAfterRemovingABlock_DeleteByBlock,
    NumBlocksIsCorrectAfterRemovingABlock_DeleteByBlockId,
    WriteAndReadImmediately,
    WriteAndReadAfterLoading,
    WriteTwiceAndRead,
    OverwriteSameSizeAndReadImmediately,
    OverwriteSameSizeAndReadAfterLoading,
    OverwriteSmallerSizeAndReadImmediately,
    OverwriteSmallerSizeAndReadAfterLoading,
    OverwriteLargerSizeAndReadAfterLoading,
    OverwriteLargerSizeAndReadImmediately,
    OverwriteNonexistingAndReadAfterLoading,
    OverwriteNonexistingAndReadImmediately,
    CanRemoveModifiedBlock,
    ForEachBlock_zeroblocks,
    ForEachBlock_oneblock,
    ForEachBlock_twoblocks,
    ForEachBlock_threeblocks,
    ForEachBlock_doesntListRemovedBlocks_oneblock,
    ForEachBlock_doesntListRemovedBlocks_twoblocks,
    Resize_Larger_FromZero,
    Resize_Larger_FromZero_BlockIsStillUsable,
    Resize_Larger,
    Resize_Larger_BlockIsStillUsable,
    Resize_Smaller,
    Resize_Smaller_BlockIsStillUsable,
    Resize_Smaller_ToZero,
    Resize_Smaller_ToZero_BlockIsStillUsable
    //TODO Just disabled because gtest doesn't allow more template parameters. Fix and reenable!
    //     see https://github.com/google/googletest/issues/1267
    //TryCreateTwoBlocksWithSameBlockIdAndSameSize,
    //TryCreateTwoBlocksWithSameBlockIdAndDifferentSize,
    //TryCreateTwoBlocksWithSameBlockIdAndFirstNullSize,
    //TryCreateTwoBlocksWithSameBlockIdAndSecondNullSize,
    //TryCreateTwoBlocksWithSameBlockIdAndBothNullSize,
);


#endif
