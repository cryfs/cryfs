#include "blockstore/implementations/versioncounting/VersionCountingBlockStore.h"
#include "blockstore/implementations/versioncounting/VersionCountingBlockStore2.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempFile.h>

using ::testing::Test;

using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::versioncounting::VersionCountingBlockStore;
using blockstore::versioncounting::VersionCountingBlockStore2;
using blockstore::versioncounting::KnownBlockVersions;
using blockstore::testfake::FakeBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::TempFile;

template<bool MissingBlockIsIntegrityViolation>
class VersionCountingBlockStoreTestFixture: public BlockStoreTestFixture {
public:
   VersionCountingBlockStoreTestFixture() :stateFile(false) {}

  TempFile stateFile;
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<VersionCountingBlockStore>(make_unique_ref<FakeBlockStore>(), stateFile.path(), 0x12345678, MissingBlockIsIntegrityViolation);
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(VersionCounting_multiclient, BlockStoreTest, VersionCountingBlockStoreTestFixture<false>);
INSTANTIATE_TYPED_TEST_CASE_P(VersionCounting_singleclient, BlockStoreTest, VersionCountingBlockStoreTestFixture<true>);

template<bool MissingBlockIsIntegrityViolation>
class VersionCountingBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  VersionCountingBlockStore2TestFixture() :stateFile(false) {}

  TempFile stateFile;
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<VersionCountingBlockStore2>(make_unique_ref<InMemoryBlockStore2>(), stateFile.path(), 0x12345678, MissingBlockIsIntegrityViolation);
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(VersionCounting_multiclient, BlockStore2Test, VersionCountingBlockStore2TestFixture<false>);
INSTANTIATE_TYPED_TEST_CASE_P(VersionCounting_singleclient, BlockStore2Test, VersionCountingBlockStore2TestFixture<true>);
