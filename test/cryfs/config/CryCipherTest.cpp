#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cryfs/config/CryCipher.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <blockstore/implementations/encrypted/EncryptedBlockStore.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/random/Random.h>

using namespace cryfs;
using namespace blockstore::encrypted;
using namespace blockstore::testfake;
using namespace blockstore;

using std::initializer_list;
using std::string;
using std::vector;
using std::find;
using boost::none;
using testing::MatchesRegex;
using namespace cpputils;

class CryCipherTest : public ::testing::Test {
public:
    void EXPECT_FINDS_CORRECT_CIPHERS(initializer_list<string> ciphers) {
      for (const string & cipher : ciphers) {
        EXPECT_FINDS_CORRECT_CIPHER(cipher);
      }
    }

    void EXPECT_FINDS_CORRECT_CIPHER(const string &cipherName) {
      EXPECT_EQ(cipherName, CryCiphers::find(cipherName).cipherName());
    }

    template<class ExpectedCipher>
    void EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE(const string &cipherName) {
        const auto &actualCipher = CryCiphers::find(cipherName);
        Data dataFixture = DataFixture::generate(1024);
        string encKey = ExpectedCipher::CreateKey(Random::PseudoRandom()).ToString();
        _EXPECT_ENCRYPTS_WITH_ACTUAL_BLOCKSTORE_DECRYPTS_CORRECTLY_WITH_EXPECTED_BLOCKSTORE<ExpectedCipher>(actualCipher, encKey, std::move(dataFixture));
    }

    template<class ExpectedCipher>
    void _EXPECT_ENCRYPTS_WITH_ACTUAL_BLOCKSTORE_DECRYPTS_CORRECTLY_WITH_EXPECTED_BLOCKSTORE(const CryCipher &actualCipher, const std::string &encKey, Data dataFixture) {
        blockstore::Key key = cpputils::Random::PseudoRandom().getFixedSize<blockstore::Key::BINARY_LENGTH>();
        Data encrypted = _encryptUsingEncryptedBlockStoreWithCipher(actualCipher, encKey, key, dataFixture.copy());
        Data decrypted = _decryptUsingEncryptedBlockStoreWithCipher<ExpectedCipher>(encKey, key, std::move(encrypted));
        EXPECT_EQ(dataFixture, decrypted);
    }

    Data _encryptUsingEncryptedBlockStoreWithCipher(const CryCipher &cipher, const std::string &encKey, const blockstore::Key &key, Data data) {
        unique_ref<FakeBlockStore> _baseStore = make_unique_ref<FakeBlockStore>();
        FakeBlockStore *baseStore = _baseStore.get();
        unique_ref<BlockStore> encryptedStore = cipher.createEncryptedBlockstore(std::move(_baseStore), encKey);
        auto created = encryptedStore->tryCreate(key, std::move(data));
        EXPECT_NE(none, created);
        return _loadBlock(baseStore, key);
    }

    template<class Cipher>
    Data _decryptUsingEncryptedBlockStoreWithCipher(const std::string &encKey, const blockstore::Key &key, Data data) {
        unique_ref<FakeBlockStore> baseStore = make_unique_ref<FakeBlockStore>();
        auto created = baseStore->tryCreate(key, std::move(data));
        EXPECT_NE(none, created);
        EncryptedBlockStore<Cipher> encryptedStore(std::move(baseStore), Cipher::EncryptionKey::FromString(encKey));
        return _loadBlock(&encryptedStore, key);
    }

    Data _loadBlock(BlockStore *store, const blockstore::Key &key) {
        auto block = store->load(key).value();
        Data data(block->size());
        std::memcpy(data.data(), block->data(), block->size());
        return data;
    }
};

TEST_F(CryCipherTest, FindsCorrectCipher) {
  EXPECT_FINDS_CORRECT_CIPHERS({
    "aes-256-gcm", "aes-256-cfb", "aes-256-gcm", "aes-256-cfb",
    "twofish-256-gcm", "twofish-256-cfb", "twofish-256-gcm", "twofish-256-cfb",
    "serpent-256-gcm", "serpent-256-cfb", "serpent-256-gcm", "serpent-256-cfb",
    "cast-256-gcm", "cast-256-cfb",
    "mars-448-gcm", "mars-448-cfb", "mars-256-gcm", "mars-256-cfb", "mars-256-gcm", "mars-256-cfb"
  });
}

TEST_F(CryCipherTest, CreatesCorrectEncryptedBlockStore) {
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<AES256_GCM>("aes-256-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<AES256_CFB>("aes-256-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<AES128_GCM>("aes-128-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<AES128_CFB>("aes-128-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Twofish256_GCM>("twofish-256-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Twofish256_CFB>("twofish-256-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Twofish128_GCM>("twofish-128-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Twofish128_CFB>("twofish-128-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Serpent256_GCM>("serpent-256-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Serpent256_CFB>("serpent-256-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Serpent128_GCM>("serpent-128-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Serpent128_CFB>("serpent-128-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Cast256_GCM>("cast-256-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Cast256_CFB>("cast-256-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Mars448_GCM>("mars-448-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Mars448_CFB>("mars-448-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Mars256_GCM>("mars-256-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Mars256_CFB>("mars-256-cfb");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Mars128_GCM>("mars-128-gcm");
    EXPECT_CREATES_CORRECT_ENCRYPTED_BLOCKSTORE<Mars128_CFB>("mars-128-cfb");
}

TEST_F(CryCipherTest, SupportedCipherNamesContainsACipher) {
  vector<string> supportedCipherNames = CryCiphers::supportedCipherNames();
  EXPECT_NE(supportedCipherNames.end(), find(supportedCipherNames.begin(), supportedCipherNames.end(), "aes-256-gcm"));
}

TEST_F(CryCipherTest, ThereIsACipherWithoutWarning) {
    EXPECT_EQ(none, CryCiphers::find("aes-256-gcm").warning());
}

TEST_F(CryCipherTest, ThereIsACipherWithIntegrityWarning) {
    EXPECT_THAT(CryCiphers::find("aes-256-cfb").warning().get(), MatchesRegex(".*integrity.*"));
}

TEST_F(CryCipherTest, EncryptionKeyHasCorrectSize_448) {
    EXPECT_EQ(Mars448_GCM::EncryptionKey::STRING_LENGTH, CryCiphers::find("mars-448-gcm").createKey(Random::PseudoRandom()).size());
}

TEST_F(CryCipherTest, EncryptionKeyHasCorrectSize_256) {
    EXPECT_EQ(AES256_GCM::EncryptionKey::STRING_LENGTH, CryCiphers::find("aes-256-gcm").createKey(Random::PseudoRandom()).size());
}

TEST_F(CryCipherTest, EncryptionKeyHasCorrectSize_128) {
    EXPECT_EQ(AES128_GCM::EncryptionKey::STRING_LENGTH, CryCiphers::find("aes-128-gcm").createKey(Random::PseudoRandom()).size());
}
