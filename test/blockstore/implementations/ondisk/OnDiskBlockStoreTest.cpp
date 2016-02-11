#include "blockstore/implementations/ondisk/OnDiskBlock.h"
#include "blockstore/implementations/ondisk/OnDiskBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStoreWithRandomKeysTest.h"
#include <gtest/gtest.h>

#include <cpp-utils/tempfile/TempDir.h>


using blockstore::BlockStore;
using blockstore::BlockStoreWithRandomKeys;
using blockstore::ondisk::OnDiskBlockStore;

using cpputils::TempDir;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class OnDiskBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  OnDiskBlockStoreTestFixture(): tempdir() {}

  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<OnDiskBlockStore>(tempdir.path());
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_CASE_P(OnDisk, BlockStoreTest, OnDiskBlockStoreTestFixture);

class OnDiskBlockStoreWithRandomKeysTestFixture: public BlockStoreWithRandomKeysTestFixture {
public:
  OnDiskBlockStoreWithRandomKeysTestFixture(): tempdir() {}
  
  unique_ref<BlockStoreWithRandomKeys> createBlockStore() override {
    return make_unique_ref<OnDiskBlockStore>(tempdir.path());
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_CASE_P(OnDisk, BlockStoreWithRandomKeysTest, OnDiskBlockStoreWithRandomKeysTestFixture);
