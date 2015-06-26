#include "../../../implementations/parallelaccess/ParallelAccessBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::parallelaccess::ParallelAccessBlockStore;
using blockstore::testfake::FakeBlockStore;

using std::unique_ptr;
using std::make_unique;
using cpputils::make_unique_ref;

class ParallelAccessBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<ParallelAccessBlockStore>(make_unique_ref<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(ParallelAccess, BlockStoreTest, ParallelAccessBlockStoreTestFixture);

//TODO Add specific tests ensuring that loading the same block twice doesn't load it twice from the underlying blockstore
