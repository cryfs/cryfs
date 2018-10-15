#include <gtest/gtest.h>
#include <blockstore/implementations/integrity/KnownBlockVersions.h>
#include <cpp-utils/tempfile/TempFile.h>

using blockstore::integrity::KnownBlockVersions;
using blockstore::BlockId;
using cpputils::TempFile;
using std::unordered_set;

class KnownBlockVersionsTest : public ::testing::Test {
public:
    KnownBlockVersionsTest() :stateFile(false), testobj(stateFile.path(), myClientId) {}

    blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
    blockstore::BlockId blockId2 = blockstore::BlockId::FromString("C772972491BB4932A1389EE14BC7090A");
    static constexpr uint32_t myClientId = 0x12345678;
    static constexpr uint32_t clientId = 0x23456789;
    static constexpr uint32_t clientId2 = 0x34567890;

    TempFile stateFile;
    KnownBlockVersions testobj;

    void setVersion(KnownBlockVersions *testobj, uint32_t clientId, const blockstore::BlockId &blockId, uint64_t version) {
        if (!testobj->checkAndUpdateVersion(clientId, blockId, version)) {
            throw std::runtime_error("Couldn't increase version");
        }
    }

    void EXPECT_VERSION_IS(uint64_t version, KnownBlockVersions *testobj, blockstore::BlockId &blockId, uint32_t clientId) {
        EXPECT_FALSE(testobj->checkAndUpdateVersion(clientId, blockId, version-1));
        EXPECT_TRUE(testobj->checkAndUpdateVersion(clientId, blockId, version+1));
    }
};

TEST_F(KnownBlockVersionsTest, setandget) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, setandget_isPerClientId) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId, 3);
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(3u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, setandget_isPerBlock) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId, blockId2, 3);
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(3u, testobj.getBlockVersion(clientId, blockId2));
}

TEST_F(KnownBlockVersionsTest, setandget_allowsIncreasing) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId, blockId, 6);
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, setandget_doesntAllowDecreasing) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_ANY_THROW(
      setVersion(&testobj, clientId, blockId, 4);
    );
}

TEST_F(KnownBlockVersionsTest, myClientId_isConsistent) {
    EXPECT_EQ(testobj.myClientId(), testobj.myClientId());
}

TEST_F(KnownBlockVersionsTest, incrementVersion_newentry) {
    auto version = testobj.incrementVersion(blockId);
    EXPECT_EQ(1u, version);
    EXPECT_EQ(1u, testobj.getBlockVersion(testobj.myClientId(), blockId));
}

TEST_F(KnownBlockVersionsTest, incrementVersion_oldentry) {
    setVersion(&testobj, testobj.myClientId(), blockId, 5);
    auto version = testobj.incrementVersion(blockId);
    EXPECT_EQ(6u, version);
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_newentry) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 5));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_sameClientSameVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 5));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_sameClientLowerVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 4));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_sameClientNewerVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_differentClientSameVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 5));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_differentClientLowerVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(3u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_differentClientHigherVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientLowerVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientSameVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 5)); // Don't allow rollback to old client's newest block, if it was superseded by another client
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientHigherVersion) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientLowerVersion_oldClientIsSelf) {
    setVersion(&testobj, testobj.myClientId(), blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(testobj.myClientId(), blockId, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(testobj.myClientId(), blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientSameVersion_oldClientIsSelf) {
    setVersion(&testobj, testobj.myClientId(), blockId, 5);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(testobj.myClientId(), blockId, 5)); // Don't allow rollback to old client's newest block, if it was superseded by another client
    EXPECT_EQ(5u, testobj.getBlockVersion(testobj.myClientId(), blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientHigherVersion_oldClientIsSelf) {
    setVersion(&testobj, testobj.myClientId(), blockId, 4);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 7));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(testobj.myClientId(), blockId, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(testobj.myClientId(), blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(clientId2, blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientLowerVersion_newClientIsSelf) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, testobj.myClientId(), blockId, 7);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 3));
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientSameVersion_newClientIsSelf) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, testobj.myClientId(), blockId, 7);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 5)); // Don't allow rollback to old client's newest block, if it was superseded by another client
    EXPECT_EQ(5u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdateVersion_oldentry_oldClientHigherVersion_newClientIsSelf) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, testobj.myClientId(), blockId, 7);
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 6));
    EXPECT_EQ(6u, testobj.getBlockVersion(clientId, blockId));
    EXPECT_EQ(7u, testobj.getBlockVersion(testobj.myClientId(), blockId));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_twoEntriesDontInfluenceEachOther_differentKeys) {
    // Setup
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId2, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 150));

    // Checks
    EXPECT_VERSION_IS(150, &testobj, blockId, clientId);
    EXPECT_VERSION_IS(100, &testobj, blockId2, clientId);
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_twoEntriesDontInfluenceEachOther_differentClientIds) {
    // Setup
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 150));

    EXPECT_VERSION_IS(150, &testobj, blockId, clientId);
    EXPECT_VERSION_IS(100, &testobj, blockId, clientId2);
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_allowsRollbackToSameClientWithSameVersionNumber) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_doesntAllowRollbackToOldClientWithSameVersionNumber) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, blockId, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId2, blockId, 10));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 100));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_empty) {
    TempFile stateFile(false);
    {
      KnownBlockVersions _1(stateFile.path(), myClientId);
    }

    EXPECT_TRUE(KnownBlockVersions(stateFile.path(), myClientId).checkAndUpdateVersion(clientId, blockId, 1));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_oneentry) {
    TempFile stateFile(false);
    EXPECT_TRUE(KnownBlockVersions(stateFile.path(), myClientId).checkAndUpdateVersion(clientId, blockId, 100));

    KnownBlockVersions obj(stateFile.path(), myClientId);
    EXPECT_EQ(100u, obj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_threeentries) {
    TempFile stateFile(false);
    {
        KnownBlockVersions obj(stateFile.path(), myClientId);
        EXPECT_TRUE(obj.checkAndUpdateVersion(obj.myClientId(), blockId, 100));
        EXPECT_TRUE(obj.checkAndUpdateVersion(obj.myClientId(), blockId2, 50));
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, blockId, 150));
    }

    KnownBlockVersions obj(stateFile.path(), myClientId);
    EXPECT_EQ(100u, obj.getBlockVersion(obj.myClientId(), blockId));
    EXPECT_EQ(50u, obj.getBlockVersion(obj.myClientId(), blockId2));
    EXPECT_EQ(150u, obj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_lastUpdateClientIdIsStored) {
    {
        KnownBlockVersions obj(stateFile.path(), myClientId);
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, blockId, 100));
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId2, blockId, 10));
    }

    KnownBlockVersions obj(stateFile.path(), myClientId);
    EXPECT_FALSE(obj.checkAndUpdateVersion(clientId, blockId, 100));
    EXPECT_TRUE(obj.checkAndUpdateVersion(clientId2, blockId, 10));
    EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, blockId, 101));
}

TEST_F(KnownBlockVersionsTest, markAsDeleted_doesntAllowReIntroducing_sameClientId) {
    setVersion(&testobj, clientId, blockId, 5);
    testobj.markBlockAsDeleted(blockId);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 5));
}

TEST_F(KnownBlockVersionsTest, markAsDeleted_doesntAllowReIntroducing_oldClientId) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId, 5);
    testobj.markBlockAsDeleted(blockId);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 5));
}

TEST_F(KnownBlockVersionsTest, markAsDeleted_checkAndUpdateDoesntDestroyState) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId, 5);
    testobj.markBlockAsDeleted(blockId);
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, blockId, 5));

    // Check block is still deleted
    EXPECT_FALSE(testobj.blockShouldExist(blockId));
}

TEST_F(KnownBlockVersionsTest, blockShouldExist_unknownBlock) {
    EXPECT_FALSE(testobj.blockShouldExist(blockId));
}

TEST_F(KnownBlockVersionsTest, blockShouldExist_knownBlock) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_TRUE(testobj.blockShouldExist(blockId));
}

TEST_F(KnownBlockVersionsTest, blockShouldExist_deletedBlock) {
    setVersion(&testobj, clientId, blockId, 5);
    testobj.markBlockAsDeleted(blockId);
    EXPECT_FALSE(testobj.blockShouldExist(blockId));
}

TEST_F(KnownBlockVersionsTest, path) {
    KnownBlockVersions obj(stateFile.path(), myClientId);
    EXPECT_EQ(stateFile.path(), obj.path());
}

TEST_F(KnownBlockVersionsTest, existingBlocks_empty) {
    EXPECT_EQ(unordered_set<BlockId>({}), testobj.existingBlocks());
}

TEST_F(KnownBlockVersionsTest, existingBlocks_oneentry) {
    setVersion(&testobj, clientId, blockId, 5);
    EXPECT_EQ(unordered_set<BlockId>({blockId}), testobj.existingBlocks());
}

TEST_F(KnownBlockVersionsTest, existingBlocks_twoentries) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId2, 5);
    EXPECT_EQ(unordered_set<BlockId>({blockId, blockId2}), testobj.existingBlocks());
}

TEST_F(KnownBlockVersionsTest, existingBlocks_twoentries_sameKey) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId, 5);
    EXPECT_EQ(unordered_set<BlockId>({blockId}), testobj.existingBlocks());
}

TEST_F(KnownBlockVersionsTest, existingBlocks_deletedEntry) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId2, 5);
    testobj.markBlockAsDeleted(blockId2);
    EXPECT_EQ(unordered_set<BlockId>({blockId}), testobj.existingBlocks());
}

TEST_F(KnownBlockVersionsTest, existingBlocks_deletedEntries) {
    setVersion(&testobj, clientId, blockId, 5);
    setVersion(&testobj, clientId2, blockId2, 5);
    testobj.markBlockAsDeleted(blockId);
    testobj.markBlockAsDeleted(blockId2);
    EXPECT_EQ(unordered_set<BlockId>({}), testobj.existingBlocks());
}
