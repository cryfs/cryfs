#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORETEST_H_

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

#include "blockstore/interface/BlockStore.h"

class MockForEachBlockCallback final
{
public:
  std::function<void(const blockstore::BlockId &)> callback()
  {
    return [this](const blockstore::BlockId &blockId)
    {
      called_with.push_back(blockId);
    };
  }

  std::vector<blockstore::BlockId> called_with;
};

class BlockStoreTestFixture
{
public:
  virtual ~BlockStoreTestFixture() {}
  virtual cpputils::unique_ref<blockstore::BlockStore> createBlockStore() = 0;
};

template <class ConcreteBlockStoreTestFixture>
class BlockStoreTest : public ::testing::Test
{
public:
  BlockStoreTest() : fixture() {}

  BOOST_STATIC_ASSERT_MSG(
      (std::is_base_of<BlockStoreTestFixture, ConcreteBlockStoreTestFixture>::value),
      "Given test fixture for instantiating the (type parameterized) BlockStoreTest must inherit from BlockStoreTestFixture");

  ConcreteBlockStoreTestFixture fixture;

  void TestBlockIsUsable(cpputils::unique_ref<blockstore::Block> block, blockstore::BlockStore *blockStore)
  {
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

  template <class Entry>
  void EXPECT_UNORDERED_EQ(const std::vector<Entry> &expected, std::vector<Entry> actual)
  {
    EXPECT_EQ(expected.size(), actual.size());
    for (const Entry &expectedEntry : expected)
    {
      removeOne(&actual, expectedEntry);
    }
  }

  template <class Entry>
  void removeOne(std::vector<Entry> *entries, const Entry &toRemove)
  {
    auto found = std::find(entries->begin(), entries->end(), toRemove);
    if (found != entries->end())
    {
      entries->erase(found);
      return;
    }
    EXPECT_TRUE(false);
  }
};

TYPED_TEST_SUITE_P(BlockStoreTest);

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
                            // TODO Just disabled because gtest doesn't allow more template parameters. Fix and reenable!
                            //      see https://github.com/google/googletest/issues/1267
                            // TryCreateTwoBlocksWithSameBlockIdAndSameSize,
                            // TryCreateTwoBlocksWithSameBlockIdAndDifferentSize,
                            // TryCreateTwoBlocksWithSameBlockIdAndFirstNullSize,
                            // TryCreateTwoBlocksWithSameBlockIdAndSecondNullSize,
                            // TryCreateTwoBlocksWithSameBlockIdAndBothNullSize,
);

#endif
