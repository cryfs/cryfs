#include "blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "blockstore/interface/BlockStore.h"
#include <gtest/gtest.h>


using blockstore::BlockStore;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

class LowToHighLevelBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  LowToHighLevelBlockStoreTestFixture() {}

  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(make_unique_ref<InMemoryBlockStore2>());
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(LowToHighLevel, BlockStoreTest, LowToHighLevelBlockStoreTestFixture);
