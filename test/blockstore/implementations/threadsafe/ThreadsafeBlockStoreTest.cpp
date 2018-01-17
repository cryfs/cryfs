#include "blockstore/implementations/threadsafe/ThreadsafeBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>


using blockstore::BlockStore;
using blockstore::threadsafe::ThreadsafeBlockStore;
using blockstore::testfake::FakeBlockStore;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

class ThreadsafeBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  ThreadsafeBlockStoreTestFixture() {}

  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<ThreadsafeBlockStore>(make_unique_ref<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Threadsafe, BlockStoreTest, ThreadsafeBlockStoreTestFixture);
