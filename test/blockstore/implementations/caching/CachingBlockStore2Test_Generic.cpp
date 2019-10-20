#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include "blockstore/implementations/caching/CachingBlockStore2.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
#include <gtest/gtest.h>


using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::caching::CachingBlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

class CachingBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(
        make_unique_ref<CachingBlockStore2>(make_unique_ref<InMemoryBlockStore2>())
    );
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Caching2, BlockStoreTest, CachingBlockStoreTestFixture);


class CachingBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<CachingBlockStore2>(make_unique_ref<InMemoryBlockStore2>());
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Caching, BlockStore2Test, CachingBlockStore2TestFixture);
