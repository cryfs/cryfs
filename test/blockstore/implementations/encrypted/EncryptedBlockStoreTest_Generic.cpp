#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/crypto/symmetric/Cipher.h>
#include "blockstore/implementations/encrypted/EncryptedBlockStore.h"
#include "blockstore/implementations/encrypted/EncryptedBlockStore2.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "../../testutils/BlockStoreTest.h"
#include "../../testutils/BlockStore2Test.h"
//TODO Move FakeAuthenticatedCipher out of test folder to normal folder. Dependencies should not point into tests of other modules.
#include "../../../cpp-utils/crypto/symmetric/testutils/FakeAuthenticatedCipher.h"
#include <gtest/gtest.h>

using ::testing::Test;

using blockstore::BlockStore;
using blockstore::BlockStore2;
using blockstore::encrypted::EncryptedBlockStore;
using blockstore::encrypted::EncryptedBlockStore2;
using blockstore::testfake::FakeBlockStore;
using blockstore::inmemory::InMemoryBlockStore2;
using cpputils::AES256_GCM;
using cpputils::AES256_CFB;
using cpputils::FakeAuthenticatedCipher;

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

template<class Cipher>
class EncryptedBlockStore2TestFixture: public BlockStore2TestFixture {
public:
  unique_ref<BlockStore2> createBlockStore() override {
    return make_unique_ref<EncryptedBlockStore2<Cipher>>(make_unique_ref<InMemoryBlockStore2>(), createKeyFixture());
  }

private:
  static typename Cipher::EncryptionKey createKeyFixture(int seed = 0) {
    Data data = DataFixture::generate(Cipher::EncryptionKey::BINARY_LENGTH, seed);
    return Cipher::EncryptionKey::FromBinary(data.data());
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(Encrypted_FakeCipher, BlockStore2Test, EncryptedBlockStore2TestFixture<FakeAuthenticatedCipher>);
INSTANTIATE_TYPED_TEST_CASE_P(Encrypted_AES256_GCM, BlockStore2Test, EncryptedBlockStore2TestFixture<AES256_GCM>);
INSTANTIATE_TYPED_TEST_CASE_P(Encrypted_AES256_CFB, BlockStore2Test, EncryptedBlockStore2TestFixture<AES256_CFB>);
