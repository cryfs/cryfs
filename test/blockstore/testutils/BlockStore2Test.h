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
};

TYPED_TEST_CASE_P(BlockStore2Test);

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  EXPECT_FALSE(this->blockStore->tryCreate(key, cpputils::Data(1024)).get());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_thenSucceeds) {
  blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(this->blockStore->tryCreate(key, cpputils::Data(1024)).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_thenSucceeds) {
  this->blockStore->create(cpputils::Data(512));
  blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_TRUE(this->blockStore->tryCreate(key, cpputils::Data(1024)).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenLoadExistingBlock_thenSucceeds) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  EXPECT_NE(boost::none, this->blockStore->load(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringExistingBlock_thenSucceeds) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  this->blockStore->store(key, cpputils::Data(1024)).wait();
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringNonexistingBlock_thenSucceeds) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(key, cpputils::Data(1024)).wait();
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringNonexistingBlock_thenSucceeds) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(key, cpputils::Data(1024)).wait();
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCreatingTwoBlocks_thenTheyGetDifferentKeys) {
  blockstore::Key key1 = this->blockStore->create(cpputils::Data(1024)).get();
  blockstore::Key key2 = this->blockStore->create(cpputils::Data(1024)).get();
  EXPECT_NE(key1, key2);
}

TYPED_TEST_P(BlockStore2Test, givenOtherwiseEmptyBlockStore_whenRemovingBlock_thenBlockIsNotLoadableAnymore) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  EXPECT_NE(boost::none, this->blockStore->load(key).get());
  this->blockStore->remove(key).get();
  EXPECT_EQ(boost::none, this->blockStore->load(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenRemovingBlock_thenBlockIsNotLoadableAnymore) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  this->blockStore->create(cpputils::Data(512));
  EXPECT_NE(boost::none, this->blockStore->load(key).get());
  this->blockStore->remove(key).get();
  EXPECT_EQ(boost::none, this->blockStore->load(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenOtherwiseEmptyBlockStore_whenRemovingExistingBlock_thenSucceeds) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  EXPECT_EQ(true, this->blockStore->remove(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenRemovingExistingBlock_thenSucceeds) {
  blockstore::Key key = this->blockStore->create(cpputils::Data(1024)).get();
  this->blockStore->create(cpputils::Data(512));
  EXPECT_EQ(true, this->blockStore->remove(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  auto result = this->blockStore->remove(key).get();
  EXPECT_EQ(false, result);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads) {
  auto key = this->blockStore->create(cpputils::Data(0)).get();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  auto key = this->blockStore->create(cpputils::Data(0)).get();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads) {
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  auto key = this->blockStore->create(data.copy()).get();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(loaded, data);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  auto key = this->blockStore->create(data.copy()).get();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(loaded, data);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(key, cpputils::Data(0)).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  this->blockStore->store(key, cpputils::Data(0)).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(key, data.copy()).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512));
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(key, data.copy()).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads) {
  auto key = this->blockStore->create(cpputils::Data(512)).get();
  this->blockStore->store(key, cpputils::Data(0)).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512)).get();
  auto key = this->blockStore->create(cpputils::Data(512)).get();
  this->blockStore->store(key, cpputils::Data(0)).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(0u, loaded.size());
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads) {
  auto key = this->blockStore->create(cpputils::Data(512)).get();
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(key, data.copy()).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads) {
  this->blockStore->create(cpputils::Data(512)).get();
  auto key = this->blockStore->create(cpputils::Data(512)).get();
  cpputils::Data data = cpputils::DataFixture::generate(1024);
  this->blockStore->store(key, data.copy()).wait();
  auto loaded = this->blockStore->load(key).get().value();
  EXPECT_EQ(data, loaded);
}

TYPED_TEST_P(BlockStore2Test, givenEmptyBlockStore_whenLoadingNonExistingBlock_thenFails) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(key).get());
}

TYPED_TEST_P(BlockStore2Test, givenNonEmptyBlockStore_whenLoadingNonExistingBlock_thenFails) {
  this->blockStore->create(cpputils::Data(512)).get();
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(boost::none, this->blockStore->load(key).get());
}


REGISTER_TYPED_TEST_CASE_P(BlockStore2Test,
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
  givenEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenCreatingAndLoadingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenCreatingAndLoadingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingNonExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingExistingEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenNonEmptyBlockStore_whenStoringAndLoadingExistingNonEmptyBlock_thenCorrectBlockLoads,
  givenEmptyBlockStore_whenLoadingNonExistingBlock_thenFails,
  givenNonEmptyBlockStore_whenLoadingNonExistingBlock_thenFails
);


#endif
