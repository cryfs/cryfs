#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORE2TEST_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_TESTUTILS_BLOCKSTORE2TEST_H_

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

#include "blockstore/interface/BlockStore2.h"

namespace boost {
inline void PrintTo(const optional<cpputils::Data> &, ::std::ostream *os) {
  *os << "optional<Data>";
}
}

class BlockStore2TestFixture {
public:
  virtual ~BlockStore2TestFixture() {}
  virtual cpputils::unique_ref<blockstore::BlockStore2> createBlockStore() = 0;
};

template<class ConcreteBlockStoreTestFixture>
class BlockStore2Test: public ::testing::Test {
public:
  BlockStore2Test() :fixture(), blockStore(this->fixture.createBlockStore()) {}

  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<BlockStore2TestFixture, ConcreteBlockStoreTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) BlockStoreTest must inherit from BlockStoreTestFixture"
  );

  ConcreteBlockStoreTestFixture fixture;
  cpputils::unique_ref<blockstore::BlockStore2> blockStore;

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

TYPED_TEST_SUITE_P(BlockStore2Test);

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  EXPECT_FALSE(this->blockStore->tryCreate(blockId, cpputils::Data(1024)));
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_thenSucceeds) {
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(this->blockStore->tryCreate(blockId, cpputils::Data(1024)));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_thenSucceeds) {
  this->blockStore->create(cpputils::Data(512));
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(this->blockStore->tryCreate(blockId, cpputils::Data(1024)));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenLoadExistingBlock_thenSucceeds) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  EXPECT_NE(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringExistingBlock_thenSucceeds) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  this->blockStore->store(blockId, cpputils::Data(1024));
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringNonexistingBlock_thenSucceeds) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(blockId, cpputils::Data(1024));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringNonexistingBlock_thenSucceeds) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(blockId, cpputils::Data(1024));
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCreatingTwoBlocks_thenTheyGetDifferentKeys) {
  blockstore::BlockId blockId1 = this->blockStore->create(cpputils::Data(1024));
  blockstore::BlockId blockId2 = this->blockStore->create(cpputils::Data(1024));
  EXPECT_NE(blockId1, blockId2);
}

TYPED_TEST_P(BlockStore2Test, givenOtherwiseEmptyBlockStore_whenRemovingBlock_thenBlockIsNotLoadableAnymore) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  EXPECT_NE(boost::none, this->blockStore->load(blockId));
  EXPECT_TRUE(this->blockStore->remove(blockId));
  EXPECT_EQ(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenRemovingBlock_thenBlockIsNotLoadableAnymore) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  this->blockStore->create(cpputils::Data(512));
  EXPECT_NE(boost::none, this->blockStore->load(blockId));
  EXPECT_TRUE(this->blockStore->remove(blockId));
  EXPECT_EQ(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenOtherwiseEmptyBlockStore_whenRemovingExistingBlock_thenSucceeds) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  EXPECT_TRUE(this->blockStore->remove(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenRemovingExistingBlock_thenSucceeds) {
  blockstore::BlockId blockId = this->blockStore->create(cpputils::Data(1024));
  this->blockStore->create(cpputils::Data(512));
  EXPECT_EQ(true, this->blockStore->remove(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto result = this->blockStore->remove(blockId);
  EXPECT_EQ(false, result);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails) {
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772973");
  blockstore::BlockId differentKey = blockstore::BlockId::FromString("290AC2C7097274A389EE14B91B72B493");
  ASSERT_TRUE(this->blockStore->tryCreate(blockId, cpputils::Data(1024)));
  EXPECT_EQ(false, this->blockStore->remove(differentKey));
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads) {
  auto blockId = this->blockStore->create(cpputils::Data(0));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  auto blockId = this->blockStore->create(cpputils::Data(0));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads) {
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  auto blockId = this->blockStore->create(data.copy());
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(loaded, data);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  auto blockId = this->blockStore->create(data.copy());
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(loaded, data);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenTryCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads) {
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772973");
  ASSERT_TRUE(this->blockStore->tryCreate(blockId, cpputils::Data(0)));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenTryCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads) {
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772973");
  this->blockStore->create(cpputils::Data(512));
  ASSERT_TRUE(this->blockStore->tryCreate(blockId, cpputils::Data(0)));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenTryCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads) {
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772973");
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  ASSERT_TRUE(this->blockStore->tryCreate(blockId, data.copy()));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(loaded, data);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenTryCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads) {
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772973");
  this->blockStore->create(cpputils::Data(512));
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  ASSERT_TRUE(this->blockStore->tryCreate(blockId, data.copy()));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(loaded, data);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(blockId, cpputils::Data(0));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(blockId, cpputils::Data(0));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(blockId, data.copy());
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(blockId, data.copy());
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads) {
  auto blockId = this->blockStore->create(cpputils::Data(512));
  this->blockStore->store(blockId, cpputils::Data(0));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  auto blockId = this->blockStore->create(cpputils::Data(512));
  this->blockStore->store(blockId, cpputils::Data(0));
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads) {
  auto blockId = this->blockStore->create(cpputils::Data(512));
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(blockId, data.copy());
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  auto blockId = this->blockStore->create(cpputils::Data(512));
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(blockId, data.copy());
  auto loaded = this->blockStore->load(blockId).value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenLoadingNonExistingBlock_thenFails) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenLoadingNonExistingBlock_thenFails) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(blockId));
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectOnEmptyBlockstore) {
  auto blockStore = this->fixture.createBlockStore();
  EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectAfterCreatingOneBlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(cpputils::Data(1));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectAfterRemovingTheLastBlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockStore->create(cpputils::Data(1));
  EXPECT_TRUE(blockStore->remove(blockId));
  EXPECT_EQ(0u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectAfterCreatingTwoBlocks) {
  auto blockStore = this->fixture.createBlockStore();
  blockStore->create(cpputils::Data(1));
  blockStore->create(cpputils::Data(0));
  EXPECT_EQ(2u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectAfterRemovingABlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockStore->create(cpputils::Data(1));
  blockStore->create(cpputils::Data(1));
  EXPECT_TRUE(blockStore->remove(blockId));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectAfterStoringANewBlock) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  auto blockStore = this->fixture.createBlockStore();
  blockStore->store(blockId, cpputils::Data(1));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, NumBlocksIsCorrectAfterStoringAnExistingBlock) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockStore->create(cpputils::Data(1));
  blockStore->store(blockId, cpputils::Data(1));
  EXPECT_EQ(1u, blockStore->numBlocks());
}

TYPED_TEST_P(BlockStore2Test, ForEachBlock_zeroblocks) {
  auto blockStore = this->fixture.createBlockStore();
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStore2Test, ForEachBlock_oneblock) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId = blockStore->create(cpputils::Data(1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({blockId}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStore2Test, ForEachBlock_twoblocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId1 = blockStore->create(cpputils::Data(1));
  auto blockId2 = blockStore->create(cpputils::Data(1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({blockId1, blockId2}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStore2Test, ForEachBlock_threeblocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId1 = blockStore->create(cpputils::Data(1));
  auto blockId2 = blockStore->create(cpputils::Data(1));
  auto blockId3 = blockStore->create(cpputils::Data(1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({blockId1, blockId2, blockId3}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStore2Test, ForEachBlock_doesntListRemovedBlocks_oneblock) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId1 = blockStore->create(cpputils::Data(1));
  EXPECT_TRUE(blockStore->remove(blockId1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStore2Test, ForEachBlock_doesntListRemovedBlocks_twoblocks) {
  auto blockStore = this->fixture.createBlockStore();
  auto blockId1 = blockStore->create(cpputils::Data(1));
  auto blockId2 = blockStore->create(cpputils::Data(1));
  EXPECT_TRUE(blockStore->remove(blockId1));
  MockForEachBlockCallback mockForEachBlockCallback;
  blockStore->forEachBlock(mockForEachBlockCallback.callback());
  this->EXPECT_UNORDERED_EQ({blockId2}, mockForEachBlockCallback.called_with);
}

TYPED_TEST_P(BlockStore2Test, TryCreateTwoBlocksWithSameBlockIdAndSameSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(blockStore->tryCreate(blockId, cpputils::Data(1024)));
  EXPECT_FALSE(blockStore->tryCreate(blockId, cpputils::Data(1024)));
}

TYPED_TEST_P(BlockStore2Test, TryCreateTwoBlocksWithSameBlockIdAndDifferentSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(blockStore->tryCreate(blockId, cpputils::Data(1024)));
  EXPECT_FALSE(blockStore->tryCreate(blockId, cpputils::Data(4096)));
}

TYPED_TEST_P(BlockStore2Test, TryCreateTwoBlocksWithSameBlockIdAndFirstNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(blockStore->tryCreate(blockId, cpputils::Data(0)));
  EXPECT_FALSE(blockStore->tryCreate(blockId, cpputils::Data(1024)));
}
/*
TYPED_TEST_P(BlockStore2Test, TryCreateTwoBlocksWithSameBlockIdAndSecondNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(blockStore->tryCreate(blockId, cpputils::Data(1024)));
  EXPECT_FALSE(blockStore->tryCreate(blockId, cpputils::Data(0)));
}

TYPED_TEST_P(BlockStore2Test, TryCreateTwoBlocksWithSameBlockIdAndBothNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(blockStore->tryCreate(blockId, cpputils::Data(0)));
  EXPECT_FALSE(blockStore->tryCreate(blockId, cpputils::Data(0)));
}*/

REGISTER_TYPED_TEST_SUITE_P(BlockStore2Test,
  givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails,
  givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_thenSucceeds,
  givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_thenSucceeds,
  givenNonEmptyBlockStore_whenLoadExistingBlock_thenSucceeds,
  givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
  givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
  givenNonEmptyBlockStore_whenStoringExistingBlock_thenSucceeds,
  givenEmptyBlockStore_whenStoringNonexistingBlock_thenSucceeds,
  givenNonEmptyBlockStore_whenStoringNonexistingBlock_thenSucceeds,
  givenEmptyBlockStore_whenCreatingTwoBlocks_thenTheyGetDifferentKeys,
  givenOtherwiseEmptyBlockStore_whenRemovingBlock_thenBlockIsNotLoadableAnymore,
  givenNonEmptyBlockStore_whenRemovingBlock_thenBlockIsNotLoadableAnymore,
  givenOtherwiseEmptyBlockStore_whenRemovingExistingBlock_thenSucceeds,
  givenNonEmptyBlockStore_whenRemovingExistingBlock_thenSucceeds,
  givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
  givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
  givenEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenTryCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenTryCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenTryCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenTryCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenLoadingNonExistingBlock_thenFails,
  givenNonEmptyBlockStore_whenLoadingNonExistingBlock_thenFails,
  NumBlocksIsCorrectOnEmptyBlockstore,
  NumBlocksIsCorrectAfterCreatingOneBlock,
  NumBlocksIsCorrectAfterRemovingTheLastBlock,
  NumBlocksIsCorrectAfterCreatingTwoBlocks,
  NumBlocksIsCorrectAfterRemovingABlock,
  NumBlocksIsCorrectAfterStoringANewBlock,
  NumBlocksIsCorrectAfterStoringAnExistingBlock,
  ForEachBlock_zeroblocks,
  ForEachBlock_oneblock,
  ForEachBlock_twoblocks,
  ForEachBlock_threeblocks,
  ForEachBlock_doesntListRemovedBlocks_oneblock,
  ForEachBlock_doesntListRemovedBlocks_twoblocks,
  TryCreateTwoBlocksWithSameBlockIdAndSameSize,
  TryCreateTwoBlocksWithSameBlockIdAndDifferentSize,
  TryCreateTwoBlocksWithSameBlockIdAndFirstNullSize
  //TODO Just disabled because gtest doesn't allow more template parameters. Fix and reenable!
  //     see https://github.com/google/googletest/issues/1267
  //TryCreateTwoBlocksWithSameBlockIdAndSecondNullSize,
  //TryCreateTwoBlocksWithSameBlockIdAndBothNullSize
);


#endif
