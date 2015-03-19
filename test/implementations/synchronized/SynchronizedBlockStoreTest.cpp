#include "../../../implementations/synchronized/SynchronizedBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::synchronized::SynchronizedBlockStore;
using blockstore::testfake::FakeBlockStore;

using std::unique_ptr;
using std::make_unique;

class SynchronizedBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<SynchronizedBlockStore>(make_unique<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Synchronized, BlockStoreTest, SynchronizedBlockStoreTestFixture);

//TODO Add specific tests ensuring that the access to the underlying blockstore is synchronized
