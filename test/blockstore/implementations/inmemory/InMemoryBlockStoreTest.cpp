#include "blockstore/implementations/inmemory/InMemoryBlock.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
#include "../../testutils/BlockStoreWithRandomKeysTest.h"
#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>


using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::BlockStoreWithRandomKeys;
using blockstore::inmemory::InMemoryBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class InMemoryBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<InMemoryBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(InMemory, BlockStoreTest, InMemoryBlockStoreTestFixture);

class InMemoryBlockStoreWithRandomKeysTestFixture: public BlockStoreWithRandomKeysTestFixture {
public:
  unique_ref<BlockStoreWithRandomKeys> createBlockStore() override {
    return make_unique_ref<InMemoryBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(InMemory, BlockStoreWithRandomKeysTest, InMemoryBlockStoreWithRandomKeysTestFixture);

class InMemoryBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<InMemoryBlockStore2>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(InMemory, BlockStore2Test, InMemoryBlockStore2TestFixture);
