#include <gtest/gtest.h>

#include <cryfs/impl/localstate/LocalStateMetadata.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <fstream>
#include <cpp-utils/crypto/symmetric/EncryptionKey.h>
#include <cpp-utils/data/DataFixture.h>

using cpputils::TempDir;
using cpputils::EncryptionKey;
using cpputils::DataFixture;
using cryfs::LocalStateMetadata;
using std::ofstream;

namespace {
EncryptionKey generateKey(size_t size, unsigned int seed = 1) {
    return EncryptionKey::FromString(DataFixture::generate(size, seed).ToString());
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
  const auto key = generateKey(1024);
  LocalStateMetadata::loadOrGenerate(stateDir.path(), key, false);
  LocalStateMetadata::loadOrGenerate(stateDir.path(), key, false);
}

TEST_F(LocalStateMetadataTest, encryptionKeyHash_whenLoadingWithDifferentKey_thenCrashes) {
  LocalStateMetadata::loadOrGenerate(stateDir.path(), generateKey(1024, 1), false);
  EXPECT_THROW(
    LocalStateMetadata::loadOrGenerate(stateDir.path(), generateKey(1024, 2), false),
    std::runtime_error
  );
}
