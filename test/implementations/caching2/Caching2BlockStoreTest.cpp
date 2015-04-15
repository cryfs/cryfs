#include "../../../implementations/caching2/Caching2BlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::caching2::Caching2BlockStore;
using blockstore::testfake::FakeBlockStore;

using std::unique_ptr;
using std::make_unique;

class Caching2BlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<Caching2BlockStore>(make_unique<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Caching2, BlockStoreTest, Caching2BlockStoreTestFixture);

//TODO Add specific tests for the blockstore
