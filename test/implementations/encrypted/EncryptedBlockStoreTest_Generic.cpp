#include "../../../implementations/encrypted/ciphers/ciphers.h"
#include "../../../implementations/encrypted/ciphers/Cipher.h"
#include "../../../implementations/encrypted/EncryptedBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"
#include "../../testutils/BlockStoreTest.h"
#include "testutils/FakeAuthenticatedCipher.h"
#include "google/gtest/gtest.h"

using ::testing::Test;

using blockstore::BlockStore;
using blockstore::encrypted::EncryptedBlockStore;
using blockstore::testfake::FakeBlockStore;
using blockstore::encrypted::AES256_GCM;
using blockstore::encrypted::AES256_CFB;

using cpputils::Data;
using cpputils::DataFixture;
using cpputils::make_unique_ref;
using cpputils::unique_ref;

template<class Cipher>
class EncryptedBlockStoreTestFixture: public BlockStoreTestFixture {
public:
  unique_ref<BlockStore> createBlockStore() override {
    return make_unique_ref<EncryptedBlockStore<Cipher>>(make_unique_ref<FakeBlockStore>(), createKeyFixture());
  }

private:
  static typename Cipher::EncryptionKey createKeyFixture(int seed = 0) {
    Data data = DataFixture::generate(Cipher::EncryptionKey::BINARY_LENGTH, seed);
    return Cipher::EncryptionKey::FromBinary(data.data());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Encrypted_FakeCipher, BlockStoreTest, EncryptedBlockStoreTestFixture<FakeAuthenticatedCipher>);
INSTANTIATE_TYPED_TEST_CASE_P(Encrypted_AES256_GCM, BlockStoreTest, EncryptedBlockStoreTestFixture<AES256_GCM>);
INSTANTIATE_TYPED_TEST_CASE_P(Encrypted_AES256_CFB, BlockStoreTest, EncryptedBlockStoreTestFixture<AES256_CFB>);
