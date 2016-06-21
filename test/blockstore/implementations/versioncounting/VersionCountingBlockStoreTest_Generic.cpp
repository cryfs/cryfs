#include "blockstore/implementations/versioncounting/VersionCountingBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include <gtest/gtest.h>

using ::testing::Test;

using blockstore::BlockStore;
using blockstore::versioncounting::VersionCountingBlockStore;
using blockstore::versioncounting::KnownBlockVersions;
using blockstore::testfake::FakeBlockStore;

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;

class VersionCountingBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<VersionCountingBlockStore>(make_unique_ref<FakeBlockStore>(), KnownBlockVersions());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(VersionCounting, BlockStoreTest, VersionCountingBlockStoreTestFixture);
