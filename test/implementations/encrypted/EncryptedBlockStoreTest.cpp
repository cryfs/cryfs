#include "../../../implementations/encrypted/EncryptedBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::encrypted::EncryptedBlockStore;
using blockstore::testfake::FakeBlockStore;

using std::unique_ptr;
using std::make_unique;

class EncryptedBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<EncryptedBlockStore>(make_unique<FakeBlockStore>());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Encrypted, BlockStoreTest, EncryptedBlockStoreTestFixture);

//TODO Add specific tests, for example loading it with a different key doesn't work
