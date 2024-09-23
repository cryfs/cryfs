#include "blockstore/implementations/mock/MockBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "blockstore/implementations/mock/MockBlock.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/interface/BlockStore.h"
#include <gtest/gtest.h>

using blockstore::BlockStore;
using blockstore::mock::MockBlockStore;
using blockstore::testfake::FakeBlockStore;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

class MockBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<MockBlockStore>(make_unique_ref<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Mock, BlockStoreTest, MockBlockStoreTestFixture);
