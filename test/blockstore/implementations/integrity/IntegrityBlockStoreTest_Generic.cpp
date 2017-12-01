#include "blockstore/implementations/integrity/IntegrityBlockStore2.h"
#include "blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
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

template<bool NoIntegrityChecks, bool MissingBlockIsIntegrityViolation>
class IntegrityBlockStoreTestFixture: public BlockStoreTestFixture {
public:
   IntegrityBlockStoreTestFixture() :stateFile(false) {}

  TempFile stateFile;
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(
        make_unique_ref<IntegrityBlockStore2>(make_unique_ref<InMemoryBlockStore2>(), stateFile.path(), 0x12345678, NoIntegrityChecks, MissingBlockIsIntegrityViolation)
    );
  }
};

using IntegrityBlockStoreTestFixture_multiclient = IntegrityBlockStoreTestFixture<false, false>;
using IntegrityBlockStoreTestFixture_singleclient = IntegrityBlockStoreTestFixture<false, true>;
using IntegrityBlockStoreTestFixture_multiclient_noIntegrityChecks = IntegrityBlockStoreTestFixture<true, false>;
using IntegrityBlockStoreTestFixture_singleclient_noIntegrityChecks = IntegrityBlockStoreTestFixture<true, true>;

INSTANTIATE_TYPED_TEST_CASE_P(Integrity_multiclient, BlockStoreTest, IntegrityBlockStoreTestFixture_multiclient);
INSTANTIATE_TYPED_TEST_CASE_P(Integrity_singleclient, BlockStoreTest, IntegrityBlockStoreTestFixture_singleclient);
INSTANTIATE_TYPED_TEST_CASE_P(Integrity_multiclient_noIntegrityChecks, BlockStoreTest, IntegrityBlockStoreTestFixture_multiclient_noIntegrityChecks);
INSTANTIATE_TYPED_TEST_CASE_P(Integrity_singleclient_noIntegrityChecks, BlockStoreTest, IntegrityBlockStoreTestFixture_singleclient_noIntegrityChecks);

template<bool NoIntegrityChecks, bool MissingBlockIsIntegrityViolation>
class IntegrityBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  IntegrityBlockStore2TestFixture() :stateFile(false) {}

  TempFile stateFile;
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<IntegrityBlockStore2>(make_unique_ref<InMemoryBlockStore2>(), stateFile.path(), 0x12345678, NoIntegrityChecks, MissingBlockIsIntegrityViolation);
  }
};

using IntegrityBlockStore2TestFixture_multiclient = IntegrityBlockStore2TestFixture<false, false>;
using IntegrityBlockStore2TestFixture_singleclient = IntegrityBlockStore2TestFixture<false, true>;
using IntegrityBlockStore2TestFixture_multiclient_noIntegrityChecks = IntegrityBlockStore2TestFixture<true, false>;
using IntegrityBlockStore2TestFixture_singleclient_noIntegrityChecks = IntegrityBlockStore2TestFixture<true, true>;

INSTANTIATE_TYPED_TEST_CASE_P(Integrity_multiclient, BlockStore2Test, IntegrityBlockStore2TestFixture_multiclient);
INSTANTIATE_TYPED_TEST_CASE_P(Integrity_singleclient, BlockStore2Test, IntegrityBlockStore2TestFixture_singleclient);
INSTANTIATE_TYPED_TEST_CASE_P(Integrity_multiclient_noIntegrityChecks, BlockStore2Test, IntegrityBlockStore2TestFixture_multiclient_noIntegrityChecks);
INSTANTIATE_TYPED_TEST_CASE_P(Integrity_singleclient_noIntegrityChecks, BlockStore2Test, IntegrityBlockStore2TestFixture_singleclient_noIntegrityChecks);
