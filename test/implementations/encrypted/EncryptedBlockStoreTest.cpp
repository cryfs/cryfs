#include "../../../implementations/encrypted/ciphers/AES256_GCM.h"
#include "../../../implementations/encrypted/EncryptedBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "google/gtest/gtest.h"


using blockstore::BlockStore;
using blockstore::encrypted::EncryptedBlockStore;
using blockstore::testfake::FakeBlockStore;
using blockstore::encrypted::AES256_GCM;

using std::unique_ptr;
using std::make_unique;

class EncryptedBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ptr<BlockStore> createBlockStore() override {
    return make_unique<EncryptedBlockStore<AES256_GCM>>(make_unique<FakeBlockStore>(), AES256_GCM::EncryptionKey::FromString("1491BB4932A389EE14BC7090A272EE5517627CFA147A971A8E6E747E0C772972"));
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Encrypted, BlockStoreTest, EncryptedBlockStoreTestFixture);

//TODO Add specific tests, for example
// - loading it with a different encKey doesn't work
// - loading it with a different blockstore::Key will fail (because it stores its key in a header)
// - when using an authenticated cipher, loading a modified block will fail
