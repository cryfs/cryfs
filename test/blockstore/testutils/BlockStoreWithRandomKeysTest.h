#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_TESTUTILS_BLOCKSTOREWITHRANDOMKEYSTEST_H_
#define MESSMER_BLOCKSTORE_TEST_TESTUTILS_BLOCKSTOREWITHRANDOMKEYSTEST_H_

#include <gtest/gtest.h>

#include "blockstore/interface/BlockStore.h"

class BlockStoreWithRandomKeysTestFixture {
public:
  virtual ~BlockStoreWithRandomKeysTestFixture() {}
  virtual cpputils::unique_ref<blockstore::BlockStoreWithRandomKeys> createBlockStore() = 0;
};

template<class ConcreteBlockStoreWithRandomKeysTestFixture>
class BlockStoreWithRandomKeysTest: public ::testing::Test {
public:
  BlockStoreWithRandomKeysTest(): fixture() {}

  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<BlockStoreWithRandomKeysTestFixture, ConcreteBlockStoreWithRandomKeysTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) BlockStoreWithRandomKeysTest must inherit from BlockStoreWithRandomKeysTestFixture"
  );

  blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");

  const std::vector<size_t> SIZES = {0, 1, 1024, 4096, 10*1024*1024};

  ConcreteBlockStoreWithRandomKeysTestFixture fixture;
};

TYPED_TEST_CASE_P(BlockStoreWithRandomKeysTest);

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndSameSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->tryCreate(this->key, cpputils::Data(1024));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(this->key, cpputils::Data(1024));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndDifferentSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->tryCreate(this->key, cpputils::Data(1024));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(this->key, cpputils::Data(4096));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndFirstNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->tryCreate(this->key, cpputils::Data(0));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(this->key, cpputils::Data(1024));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndSecondNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->tryCreate(this->key, cpputils::Data(1024));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(this->key, cpputils::Data(0));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndBothNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->tryCreate(this->key, cpputils::Data(0));
  (*block)->flush(); //TODO Ideally, flush shouldn't be necessary here.
  auto block2 = blockStore->tryCreate(this->key, cpputils::Data(0));
  EXPECT_NE(boost::none, block);
  EXPECT_EQ(boost::none, block2);
}

REGISTER_TYPED_TEST_CASE_P(BlockStoreWithRandomKeysTest,
  CreateTwoBlocksWithSameKeyAndSameSize,
  CreateTwoBlocksWithSameKeyAndDifferentSize,
  CreateTwoBlocksWithSameKeyAndFirstNullSize,
  CreateTwoBlocksWithSameKeyAndSecondNullSize,
  CreateTwoBlocksWithSameKeyAndBothNullSize
);


#endif
