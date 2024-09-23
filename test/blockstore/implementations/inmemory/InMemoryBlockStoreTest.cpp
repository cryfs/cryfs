#include "../../testutils/BlockStore2Test.h"
#include "../../testutils/BlockStoreTest.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h"
#include "blockstore/interface/BlockStore.h"
#include "blockstore/interface/BlockStore2.h"
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include <gtest/gtest.h>


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

class InMemoryBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<InMemoryBlockStore2>();
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(InMemory, BlockStore2Test, InMemoryBlockStore2TestFixture);
