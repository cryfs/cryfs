#include "../../../implementations/testfake/FakeBlock.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStoreWithRandomKeysTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::BlockStoreWithRandomKeys;
using blockstore::testfake::FakeBlockStore;

using std::unique_ptr;
using std::make_unique;

class FakeBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<FakeBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(TestFake, BlockStoreTest, FakeBlockStoreTestFixture);

class FakeBlockStoreWithRandomKeysTestFixture: public BlockStoreWithRandomKeysTestFixture {
public:
  unique_ptr<BlockStoreWithRandomKeys> createBlockStore() override {
    return make_unique<FakeBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(TestFake, BlockStoreWithRandomKeysTest, FakeBlockStoreWithRandomKeysTestFixture);
