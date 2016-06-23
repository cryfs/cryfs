#include <gtest/gtest.h>
#include <blockstore/implementations/versioncounting/KnownBlockVersions.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <blockstore/implementations/versioncounting/VersionCountingBlock.h>

using blockstore::versioncounting::KnownBlockVersions;
using blockstore::versioncounting::VersionCountingBlock;
using cpputils::TempFile;

class KnownBlockVersionsTest : public ::testing::Test {
public:
    KnownBlockVersionsTest() :stateFile(false), testobj(stateFile.path()) {}

    blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
    blockstore::Key key2 = blockstore::Key::FromString("C772972491BB4932A1389EE14BC7090A");
    uint32_t clientId = 0x12345678;
    uint32_t clientId2 = 0x23456789;

    TempFile stateFile;
    KnownBlockVersions testobj;

    void EXPECT_VERSION_IS(uint64_t version, KnownBlockVersions *testobj, blockstore::Key &key, uint32_t clientId) {
        EXPECT_FALSE(testobj->checkAndUpdateVersion(clientId, key, version-1));
        EXPECT_TRUE(testobj->checkAndUpdateVersion(clientId, key, version+1));
    }
};

TEST_F(KnownBlockVersionsTest, setandget) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, setandget_isPerClientId) {
    testobj.setVersion(clientId, key, 5);
    testobj.setVersion(clientId2, key, 3);
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(3u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, setandget_isPerBlock) {
    testobj.setVersion(clientId, key, 5);
    testobj.setVersion(clientId, key2, 3);
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(3u, testobj.getBlockVersion(clientId, key2));
}

TEST_F(KnownBlockVersionsTest, setandget_allowsIncreasing) {
    testobj.setVersion(clientId, key, 5);
    testobj.setVersion(clientId, key, 6);
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, setandget_doesntAllowDecreasing) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_ANY_THROW(
      testobj.setVersion(clientId, key, 4);
    );
}

TEST_F(KnownBlockVersionsTest, myClientId_isConsistent) {
    EXPECT_EQ(testobj.myClientId(), testobj.myClientId());
}

TEST_F(KnownBlockVersionsTest, incrementVersion_newentry_versionzero) {
    auto version = testobj.incrementVersion(key, VersionCountingBlock::VERSION_ZERO);
    EXPECT_EQ(1u, version);
    EXPECT_EQ(1u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, incrementVersion_newentry_versionnotzero) {
    auto version = testobj.incrementVersion(key, 5);
    EXPECT_EQ(6u, version);
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, incrementVersion_oldentry_sameVersion) {
    testobj.setVersion(testobj.myClientId(), key, 5);
    auto version = testobj.incrementVersion(key, 5);
    EXPECT_EQ(6u, version);
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, incrementVersion_oldentry_lowerVersion1) {
    testobj.setVersion(testobj.myClientId(), key, 5);
    auto version = testobj.incrementVersion(key, 4);
    EXPECT_EQ(6u, version);
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, incrementVersion_oldentry_lowerVersion2) {
    testobj.setVersion(testobj.myClientId(), key, 5);
    auto version = testobj.incrementVersion(key, 3);
    EXPECT_EQ(6u, version);
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, incrementVersion_oldentry_higherVersion) {
    testobj.setVersion(testobj.myClientId(), key, 5);
    auto version = testobj.incrementVersion(key, 6);
    EXPECT_EQ(7u, version);
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_newentry) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 5));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_sameClientSameVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 5));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_sameClientLowerVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 4));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_sameClientNewerVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_differentClientSameVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 5));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_differentClientLowerVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(3u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_differentClientHigherVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientLowerVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientSameVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 5)); // Don't allow rollback to old client's newest block, if it was superseded by another client
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientHigherVersion) {
    testobj.setVersion(clientId, key, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientLowerVersion_oldClientIsSelf) {
    testobj.incrementVersion(key, 4);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(testobj.myClientId(), key, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(testobj.myClientId(), key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientSameVersion_oldClientIsSelf) {
    testobj.incrementVersion(key, 4);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(testobj.myClientId(), key, 5)); // Don't allow rollback to old client's newest block, if it was superseded by another client
    EXPECT_EQ(5u, testobj.getBlockVersion(testobj.myClientId(), key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientHigherVersion_oldClientIsSelf) {
    testobj.incrementVersion(key, 4);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 7));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(testobj.myClientId(), key, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), key));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientLowerVersion_newClientIsSelf) {
    testobj.setVersion(clientId, key, 5);
    testobj.incrementVersion(key, 6);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientSameVersion_newClientIsSelf) {
    testobj.setVersion(clientId, key, 5);
    testobj.incrementVersion(key, 6);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 5)); // Don't allow rollback to old client's newest block, if it was superseded by another client
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientHigherVersion_newClientIsSelf) {
    testobj.setVersion(clientId, key, 5);
    testobj.incrementVersion(key, 6);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, key));
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), key));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_twoEntriesDontInfluenceEachOther_differentKeys) {
    // Setup
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key2, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 150));

    // Checks
    EXPECT_VERSION_IS(150, &testobj, key, clientId);
    EXPECT_VERSION_IS(100, &testobj, key2, clientId);
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_twoEntriesDontInfluenceEachOther_differentClientIds) {
    // Setup
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 150));

    EXPECT_VERSION_IS(150, &testobj, key, clientId);
    EXPECT_VERSION_IS(100, &testobj, key, clientId2);
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_allowsRollbackToSameClientWithSameVersionNumber) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_doesntAllowRollbackToOldClientWithSameVersionNumber) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, key, 10));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 100));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_empty) {
    TempFile stateFile(false);
    KnownBlockVersions(stateFile.path());

    EXPECT_TRUE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(clientId, key, 1));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_oneentry) {
    TempFile stateFile(false);
    EXPECT_TRUE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(clientId, key, 100));

    KnownBlockVersions obj(stateFile.path());
    EXPECT_EQ(100u, obj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_threeentries) {
    TempFile stateFile(false);
    {
        KnownBlockVersions obj(stateFile.path());
        EXPECT_TRUE(obj.checkAndUpdateVersion(obj.myClientId(), key, 100));
        EXPECT_TRUE(obj.checkAndUpdateVersion(obj.myClientId(), key2, 50));
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, key, 150));
    }

    KnownBlockVersions obj(stateFile.path());
    EXPECT_EQ(100u, obj.getBlockVersion(obj.myClientId(), key));
    EXPECT_EQ(50u, obj.getBlockVersion(obj.myClientId(), key2));
    EXPECT_EQ(150u, obj.getBlockVersion(clientId, key));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_lastUpdateClientIdIsStored) {
    {
        KnownBlockVersions obj(stateFile.path());
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, key, 100));
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId2, key, 10));
    }

    KnownBlockVersions obj(stateFile.path());
    EXPECT_FALSE(obj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(obj.checkAndUpdateVersion(clientId2, key, 10));
    EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, key, 101));
}
