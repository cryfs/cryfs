#include "../../../implementations/caching/CachingBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::caching::CachingBlockStore;
using blockstore::testfake::FakeBlockStore;

using std::unique_ptr;
using std::make_unique;

class CachingBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<CachingBlockStore>(make_unique<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Caching, BlockStoreTest, CachingBlockStoreTestFixture);

//TODO Add specific tests for the blockstore
