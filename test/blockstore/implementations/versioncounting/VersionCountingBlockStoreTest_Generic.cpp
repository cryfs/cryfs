#include "blockstore/implementations/versioncounting/VersionCountingBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempFile.h>

using ::testing::Test;

using blockstore::BlockStore;
using blockstore::versioncounting::VersionCountingBlockStore;
using blockstore::versioncounting::KnownBlockVersions;
using blockstore::testfake::FakeBlockStore;

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::TempFile;

class VersionCountingBlockStoreTestFixture: public BlockStoreTestFixture {
public:
   VersionCountingBlockStoreTestFixture() :stateFile(false) {}

  TempFile stateFile;
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<VersionCountingBlockStore>(make_unique_ref<FakeBlockStore>(), stateFile.path());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(VersionCounting, BlockStoreTest, VersionCountingBlockStoreTestFixture);
