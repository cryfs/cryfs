#include "blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>


using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class InMemoryBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(
        make_unique_ref<InMemoryBlockStore2>()
    );
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(InMemory, BlockStoreTest, InMemoryBlockStoreTestFixture);
