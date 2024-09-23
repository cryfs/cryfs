#include "../../testutils/BlockStore2Test.h"
#include "../../testutils/BlockStoreTest.h"
#include "blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h"
#include "blockstore/implementations/ondisk/OnDiskBlockStore2.h"
#include "blockstore/interface/BlockStore.h"
#include "blockstore/interface/BlockStore2.h"
#include <gtest/gtest.h>

#include <cpp-utils/tempfile/TempDir.h>


using blockstore::BlockStore;
using blockstore::ondisk::OnDiskBlockStore2;
using blockstore::BlockStore2;
using blockstore::lowtohighlevel::LowToHighLevelBlockStore;

using cpputils::TempDir;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class OnDiskBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  OnDiskBlockStoreTestFixture(): tempdir() {}

  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<LowToHighLevelBlockStore>(
      make_unique_ref<OnDiskBlockStore2>(tempdir.path())
    );
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_SUITE_P(OnDisk, BlockStoreTest, OnDiskBlockStoreTestFixture);

class OnDiskBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  OnDiskBlockStore2TestFixture(): tempdir() {}

  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<OnDiskBlockStore2>(tempdir.path());
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_SUITE_P(OnDisk, BlockStore2Test, OnDiskBlockStore2TestFixture);
