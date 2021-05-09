#include "blockstore/implementations/rustbridge/RustBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
#include <gtest/gtest.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include <cpp-utils/tempfile/TempFile.h>

using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::rust::RustBlockStore2;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::TempFile;

class RustBridgeIntegrityInMemoryBlockStoreTestFixture : public BlockStoreTestFixture
{
public:
    RustBridgeIntegrityInMemoryBlockStoreTestFixture() :stateFile(false) {}

    TempFile stateFile;
    unique_ref<BlockStore> createBlockStore() override
    {
        return make_unique_ref<LowToHighLevelBlockStore>(
            make_unique_ref<RustBlockStore2>(
                blockstore::rust::bridge::new_integrity_inmemory_blockstore(stateFile.path().c_str())));
    }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Rust_IntegrityInMemory, BlockStoreTest, RustBridgeIntegrityInMemoryBlockStoreTestFixture);

class RustBridgeIntegrityInMemoryBlockStore2TestFixture : public BlockStore2TestFixture
{
public:
    RustBridgeIntegrityInMemoryBlockStore2TestFixture() :stateFile(false) {}

    TempFile stateFile;
    unique_ref<BlockStore2> createBlockStore() override
    {
        return make_unique_ref<RustBlockStore2>(
            blockstore::rust::bridge::new_integrity_inmemory_blockstore(stateFile.path().c_str()));
    }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Rust_IntegrityInMemory, BlockStore2Test, RustBridgeIntegrityInMemoryBlockStore2TestFixture);
