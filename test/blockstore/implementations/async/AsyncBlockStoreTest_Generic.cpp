#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include "blockstore/implementations/async/AsyncBlockStore2.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
#include <gtest/gtest.h>

using ::testing::Test;

using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::async::AsyncBlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;

template<size_t NUM_THREADS>
class AsyncBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(
        make_unique_ref<AsyncBlockStore2>(make_unique_ref<InMemoryBlockStore2>(), NUM_THREADS)
    );
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Async_OneThread, BlockStoreTest, AsyncBlockStoreTestFixture<1>);
INSTANTIATE_TYPED_TEST_CASE_P(Async_TenThreads, BlockStoreTest, AsyncBlockStoreTestFixture<10>);

template<size_t NUM_THREADS>
class AsyncBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<AsyncBlockStore2>(make_unique_ref<InMemoryBlockStore2>(), NUM_THREADS);
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Async_OneThread, BlockStore2Test, AsyncBlockStore2TestFixture<1>);
INSTANTIATE_TYPED_TEST_CASE_P(Async_TenThreads, BlockStore2Test, AsyncBlockStore2TestFixture<10>);
