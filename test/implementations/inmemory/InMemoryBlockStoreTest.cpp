#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <messmer/blockstore/test/testutils/BlockStoreTest.h>
#include <messmer/blockstore/test/testutils/BlockStoreWithRandomKeysTest.h>
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::BlockStoreWithRandomKeys;
using blockstore::inmemory::InMemoryBlockStore;

using std::unique_ptr;
using std::make_unique;

class InMemoryBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<InMemoryBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(InMemory, BlockStoreTest, InMemoryBlockStoreTestFixture);

class InMemoryBlockStoreWithRandomKeysTestFixture: public BlockStoreWithRandomKeysTestFixture {
public:
  unique_ptr<BlockStoreWithRandomKeys> createBlockStore() override {
    return make_unique<InMemoryBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(InMemory, BlockStoreWithRandomKeysTest, InMemoryBlockStoreWithRandomKeysTestFixture);
