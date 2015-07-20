#include "../../../implementations/inmemory/InMemoryBlock.h"
#include "../../../implementations/inmemory/InMemoryBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStoreWithRandomKeysTest.h"
#include "google/gtest/gtest.h"
#include <messmer/cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>


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
