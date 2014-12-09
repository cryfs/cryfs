#pragma once
#ifndef TEST_BLOCKSTORE_IMPLEMENTATIONS_TESTUTILS_BLOCKSTOREWITHRANDOMKEYSTEST_H_
#define TEST_BLOCKSTORE_IMPLEMENTATIONS_TESTUTILS_BLOCKSTOREWITHRANDOMKEYSTEST_H_

#include <gtest/gtest.h>

#include <blockstore/interface/BlockStore.h>
#include <test/testutils/DataBlockFixture.h>
#include "test/testutils/TempDir.h"

class BlockStoreWithRandomKeysTestFixture {
public:
  virtual std::unique_ptr<blockstore::BlockStoreWithRandomKeys> createBlockStore() = 0;
};

template<class ConcreteBlockStoreWithRandomKeysTestFixture>
class BlockStoreWithRandomKeysTest: public ::testing::Test {
public:
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
  auto block = blockStore->create(this->key, 1024);
  auto block2 = blockStore->create(this->key, 1024);
  EXPECT_TRUE((bool)block);
  EXPECT_FALSE((bool)block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndDifferentSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(this->key, 1024);
  auto block2 = blockStore->create(this->key, 4096);
  EXPECT_TRUE((bool)block);
  EXPECT_FALSE((bool)block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndFirstNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(this->key, 0);
  auto block2 = blockStore->create(this->key, 1024);
  EXPECT_TRUE((bool)block);
  EXPECT_FALSE((bool)block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndSecondNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(this->key, 1024);
  auto block2 = blockStore->create(this->key, 0);
  EXPECT_TRUE((bool)block);
  EXPECT_FALSE((bool)block2);
}

TYPED_TEST_P(BlockStoreWithRandomKeysTest, CreateTwoBlocksWithSameKeyAndBothNullSize) {
  auto blockStore = this->fixture.createBlockStore();
  auto block = blockStore->create(this->key, 0);
  auto block2 = blockStore->create(this->key, 0);
  EXPECT_TRUE((bool)block);
  EXPECT_FALSE((bool)block2);
}

REGISTER_TYPED_TEST_CASE_P(BlockStoreWithRandomKeysTest,
  CreateTwoBlocksWithSameKeyAndSameSize,
  CreateTwoBlocksWithSameKeyAndDifferentSize,
  CreateTwoBlocksWithSameKeyAndFirstNullSize,
  CreateTwoBlocksWithSameKeyAndSecondNullSize,
  CreateTwoBlocksWithSameKeyAndBothNullSize
);


#endif
