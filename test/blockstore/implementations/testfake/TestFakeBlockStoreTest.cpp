#include "../../testutils/BlockStoreTest.h"
#include "blockstore/implementations/testfake/FakeBlock.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/interface/BlockStore.h"
#include <gtest/gtest.h>

using blockstore::BlockStore;
using blockstore::testfake::FakeBlockStore;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class FakeBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<FakeBlockStore>();
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(TestFake, BlockStoreTest, FakeBlockStoreTestFixture);
