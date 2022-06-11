#include "blockstore/implementations/rustbridge/RustBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempFile.h>


using blockstore::BlockStore;
using blockstore::rust::RustBlockStore;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

class RustBridgeLockingInMemoryBlockStoreTest: public BlockStoreTestFixture {
public:
  RustBridgeLockingInMemoryBlockStoreTest() {}

  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<RustBlockStore>(
            blockstore::rust::bridge::new_locking_inmemory_blockstore());
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Rust_LockingInMemory, BlockStoreTest, RustBridgeLockingInMemoryBlockStoreTest);
