#include "blockstore/implementations/rustbridge/RustBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>

using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::rust::RustBlockStore2;
using cpputils::make_unique_ref;
using cpputils::unique_ref;

class RustBridgeInMemoryBlockStoreTestFixture : public BlockStoreTestFixture
{
public:
  unique_ref<BlockStore> createBlockStore() override
  {
    return make_unique_ref<LowToHighLevelBlockStore>(
        make_unique_ref<RustBlockStore2>(
            blockstore::rust::bridge::new_inmemory_blockstore()));
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Rust_InMemory, BlockStoreTest, RustBridgeInMemoryBlockStoreTestFixture);
