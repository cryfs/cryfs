#include "blockstore/implementations/parallelaccess/ParallelAccessBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>


using blockstore::BlockStore;
using blockstore::parallelaccess::ParallelAccessBlockStore;
using blockstore::testfake::FakeBlockStore;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

class ParallelAccessBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<ParallelAccessBlockStore>(make_unique_ref<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(ParallelAccess, BlockStoreTest, ParallelAccessBlockStoreTestFixture);

//TODO Add specific tests ensuring that loading the same block twice doesn't load it twice from the underlying blockstore
