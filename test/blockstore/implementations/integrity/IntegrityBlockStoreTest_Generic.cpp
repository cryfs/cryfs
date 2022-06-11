#include "blockstore/implementations/integrity/IntegrityBlockStore2.h"
#include "blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempFile.h>


using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::integrity::IntegrityBlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;

using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::TempFile;

template<bool AllowIntegrityViolations, bool MissingBlockIsIntegrityViolation>
class IntegrityBlockStoreTestFixture: public BlockStoreTestFixture {
public:
   IntegrityBlockStoreTestFixture() :stateFile(false) {}

  TempFile stateFile;
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(
        make_unique_ref<IntegrityBlockStore2>(make_unique_ref<InMemoryBlockStore2>(), stateFile.path(), 0x12345678, AllowIntegrityViolations, MissingBlockIsIntegrityViolation, [] {})
    );
  }
};

using IntegrityBlockStoreTestFixture_multiclient = IntegrityBlockStoreTestFixture<false, false>;
using IntegrityBlockStoreTestFixture_singleclient = IntegrityBlockStoreTestFixture<false, true>;
using IntegrityBlockStoreTestFixture_multiclient_allowIntegrityViolations = IntegrityBlockStoreTestFixture<true, false>;
using IntegrityBlockStoreTestFixture_singleclient_allowIntegrityViolations = IntegrityBlockStoreTestFixture<true, true>;

INSTANTIATE_TYPED_TEST_SUITE_P(Integrity_multiclient, BlockStoreTest, IntegrityBlockStoreTestFixture_multiclient);
INSTANTIATE_TYPED_TEST_SUITE_P(Integrity_singleclient, BlockStoreTest, IntegrityBlockStoreTestFixture_singleclient);
INSTANTIATE_TYPED_TEST_SUITE_P(Integrity_multiclient_allowIntegrityViolations, BlockStoreTest, IntegrityBlockStoreTestFixture_multiclient_allowIntegrityViolations);
INSTANTIATE_TYPED_TEST_SUITE_P(Integrity_singleclient_allowIntegrityViolations, BlockStoreTest, IntegrityBlockStoreTestFixture_singleclient_allowIntegrityViolations);
