#include <gtest/gtest.h>

#include <cryfs/localstate/LocalStateMetadata.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <fstream>

using cpputils::TempDir;
using cryfs::LocalStateMetadata;
using std::ofstream;

class LocalStateMetadataTest : public ::testing::Test {
public:
    TempDir stateDir;
    TempDir stateDir2;
};

TEST_F(LocalStateMetadataTest, myClientId_ValueIsConsistent) {
    LocalStateMetadata metadata1 = LocalStateMetadata::loadOrGenerate(stateDir.path());
    LocalStateMetadata metadata2 = LocalStateMetadata::loadOrGenerate(stateDir.path());
    EXPECT_EQ(metadata1.myClientId(), metadata2.myClientId());
}

TEST_F(LocalStateMetadataTest, myClientId_ValueIsRandomForNewClient) {
    LocalStateMetadata metadata1 = LocalStateMetadata::loadOrGenerate(stateDir.path());
    LocalStateMetadata metadata2 = LocalStateMetadata::loadOrGenerate(stateDir2.path());
    EXPECT_NE(metadata1.myClientId(), metadata2.myClientId());
}

#ifndef CRYFS_NO_COMPATIBILITY
TEST_F(LocalStateMetadataTest, myClientId_TakesLegacyValueIfSpecified) {
  ofstream file((stateDir.path() / "myClientId").native());
  file << 12345u;
  file.close();

  LocalStateMetadata metadata = LocalStateMetadata::loadOrGenerate(stateDir.path());
  EXPECT_EQ(12345u, metadata.myClientId());
}
#endif
