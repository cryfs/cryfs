#include "../../../implementations/ondisk/OnDiskBlock.h"
#include "../../../implementations/ondisk/OnDiskBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStoreWithRandomKeysTest.h"
#include "google/gtest/gtest.h"

#include "messmer/cpp-utils/tempfile/TempDir.h"


using blockstore::BlockStore;
using blockstore::BlockStoreWithRandomKeys;
using blockstore::ondisk::OnDiskBlockStore;

using std::unique_ptr;
using std::make_unique;

using cpputils::TempDir;

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
