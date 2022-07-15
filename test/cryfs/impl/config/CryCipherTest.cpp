#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cryfs/impl/config/CryCipher.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>
#include <blockstore/implementations/inmemory/InMemoryBlockStore2.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/random/Random.h>

using namespace cryfs;
using namespace blockstore::inmemory;
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

    Data _loadBlock(BlockStore2 *store, const blockstore::BlockId &blockId) {
        return store->load(blockId).value();
    }
};

TEST_F(CryCipherTest, FindsCorrectCipher) {
  EXPECT_FINDS_CORRECT_CIPHERS({
    "aes-256-gcm", "aes-256-cfb", "aes-256-gcm", "aes-256-cfb",
    "twofish-256-gcm", "twofish-256-cfb", "twofish-256-gcm", "twofish-256-cfb",
    "serpent-256-gcm", "serpent-256-cfb", "serpent-256-gcm", "serpent-256-cfb",
    "cast-256-gcm", "cast-256-cfb", "mars-448-gcm", "mars-448-cfb",
    "mars-256-gcm", "mars-256-cfb", "mars-256-gcm", "mars-256-cfb"
  });
}

TEST_F(CryCipherTest, SupportedCipherNamesContainsACipher) {
  vector<string> supportedCipherNames = CryCiphers::supportedCipherNames();
  EXPECT_NE(supportedCipherNames.end(), find(supportedCipherNames.begin(), supportedCipherNames.end(), "aes-256-gcm"));
}

TEST_F(CryCipherTest, ThereIsACipherWithoutWarning) {
    EXPECT_EQ(none, CryCiphers::find("aes-256-gcm").warning());
}

TEST_F(CryCipherTest, ThereIsACipherWithIntegrityWarning) {
    EXPECT_THAT(CryCiphers::find("aes-256-cfb").warning().value(), MatchesRegex(".*integrity.*"));
}

TEST_F(CryCipherTest, EncryptionKeyHasCorrectSize_448) {
    EXPECT_EQ(Mars448_GCM::STRING_KEYSIZE, CryCiphers::find("mars-448-gcm").createKey(Random::PseudoRandom()).size());
}

TEST_F(CryCipherTest, EncryptionKeyHasCorrectSize_256) {
    EXPECT_EQ(AES256_GCM::STRING_KEYSIZE, CryCiphers::find("aes-256-gcm").createKey(Random::PseudoRandom()).size());
}

TEST_F(CryCipherTest, EncryptionKeyHasCorrectSize_128) {
    EXPECT_EQ(AES128_GCM::STRING_KEYSIZE, CryCiphers::find("aes-128-gcm").createKey(Random::PseudoRandom()).size());
}
