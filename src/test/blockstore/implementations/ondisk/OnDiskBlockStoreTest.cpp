#include <blockstore/implementations/ondisk/OnDiskBlock.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <test/blockstore/testutils/BlockStoreWithRandomKeysTest.h>
#include <test/blockstore/testutils/BlockStoreTest.h>
#include "gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::BlockStoreWithRandomKeys;
using blockstore::ondisk::OnDiskBlockStore;

using std::unique_ptr;
using std::make_unique;

class OnDiskBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<OnDiskBlockStore>(tempdir.path());
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_CASE_P(OnDisk, BlockStoreTest, OnDiskBlockStoreTestFixture);

class OnDiskBlockStoreWithRandomKeysTestFixture: public BlockStoreWithRandomKeysTestFixture {
public:
  unique_ptr<BlockStoreWithRandomKeys> createBlockStore() override {
    return make_unique<OnDiskBlockStore>(tempdir.path());
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_CASE_P(OnDisk, BlockStoreWithRandomKeysTest, OnDiskBlockStoreWithRandomKeysTestFixture);
