#include <gtest/gtest.h>

#include <cryfs/impl/localstate/LocalStateMetadata.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <fstream>
#include <cpp-utils/crypto/symmetric/EncryptionKey.h>
#include <cpp-utils/random/Random.h>

using cpputils::TempDir;
using cpputils::EncryptionKey;
using cryfs::LocalStateMetadata;
using std::ofstream;

namespace {
EncryptionKey generateKey(unsigned int seed) {
    // Generate a deterministic key by repeating the seed byte
    std::string hex;
    for (size_t i = 0; i < 32; ++i) {
        char buf[3];
        snprintf(buf, sizeof(buf), "%02X", (seed + i) & 0xFF);
        hex += buf;
    }
    return EncryptionKey::FromString(hex);
}
}

class LocalStateMetadataTest : public ::testing::Test {
public:
    TempDir stateDir;
    TempDir stateDir2;
};

TEST_F(LocalStateMetadataTest, myClientId_ValueIsConsistent) {
    const LocalStateMetadata metadata1 = LocalStateMetadata::loadOrGenerate(stateDir.path(), EncryptionKey::Null(0), false);
    const LocalStateMetadata metadata2 = LocalStateMetadata::loadOrGenerate(stateDir.path(), EncryptionKey::Null(0), false);
    EXPECT_EQ(metadata1.myClientId(), metadata2.myClientId());
}

TEST_F(LocalStateMetadataTest, myClientId_ValueIsRandomForNewClient) {
    const LocalStateMetadata metadata1 = LocalStateMetadata::loadOrGenerate(stateDir.path(), EncryptionKey::Null(0), false);
    const LocalStateMetadata metadata2 = LocalStateMetadata::loadOrGenerate(stateDir2.path(), EncryptionKey::Null(0), false);
    EXPECT_NE(metadata1.myClientId(), metadata2.myClientId());
}

#ifndef CRYFS_NO_COMPATIBILITY
TEST_F(LocalStateMetadataTest, myClientId_TakesLegacyValueIfSpecified) {
  ofstream file((stateDir.path() / "myClientId").string());
  file << 12345u;
  file.close();

  const LocalStateMetadata metadata = LocalStateMetadata::loadOrGenerate(stateDir.path(), EncryptionKey::Null(0), false);
  EXPECT_EQ(12345u, metadata.myClientId());
}
#endif

TEST_F(LocalStateMetadataTest, encryptionKeyHash_whenLoadingWithSameKey_thenDoesntCrash) {
  const auto key = generateKey(1);
  LocalStateMetadata::loadOrGenerate(stateDir.path(), key, false);
  LocalStateMetadata::loadOrGenerate(stateDir.path(), key, false);
}

TEST_F(LocalStateMetadataTest, encryptionKeyHash_whenLoadingWithDifferentKey_thenCrashes) {
  LocalStateMetadata::loadOrGenerate(stateDir.path(), generateKey(1), false);
  EXPECT_THROW(
    LocalStateMetadata::loadOrGenerate(stateDir.path(), generateKey(2), false),
    std::runtime_error
  );
}
