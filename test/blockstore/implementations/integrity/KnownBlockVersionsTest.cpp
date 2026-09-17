#include <gtest/gtest.h>
#include <blockstore/implementations/integrity/KnownBlockVersions.h>
#include <cpp-utils/data/SerializationHelper.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <array>
#include <atomic>
#include <thread>

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

    // Deterministic, distinct block ids, so tests can work on many blocks without a random source
    static BlockId blockIdForIndex(uint64_t index) {
        std::array<unsigned char, BlockId::BINARY_LENGTH> data{};
        cpputils::serialize<uint64_t>(data.data(), index);
        return BlockId::FromBinary(data.data());
    }

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
    const TempFile stateFile(false);
    {
      const KnownBlockVersions _1(stateFile.path(), myClientId);
    }

    EXPECT_TRUE(KnownBlockVersions(stateFile.path(), myClientId).checkAndUpdateVersion(clientId, blockId, 1));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_oneentry) {
    const TempFile stateFile(false);
    EXPECT_TRUE(KnownBlockVersions(stateFile.path(), myClientId).checkAndUpdateVersion(clientId, blockId, 100));

    const KnownBlockVersions obj(stateFile.path(), myClientId);
    EXPECT_EQ(100u, obj.getBlockVersion(clientId, blockId));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_threeentries) {
    const TempFile stateFile(false);
    {
        KnownBlockVersions obj(stateFile.path(), myClientId);
        EXPECT_TRUE(obj.checkAndUpdateVersion(obj.myClientId(), blockId, 100));
        EXPECT_TRUE(obj.checkAndUpdateVersion(obj.myClientId(), blockId2, 50));
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, blockId, 150));
    }

    const KnownBlockVersions obj(stateFile.path(), myClientId);
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
    const KnownBlockVersions obj(stateFile.path(), myClientId);
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

// Regression test for a data race. markBlockAsDeleted(), blockShouldExist() and existingBlocks() used to access
// _lastUpdateClientId without taking the mutex, while incrementVersion() and checkAndUpdateVersion() modify it
// under the mutex. In CryFS, incrementVersion() runs on the cache flusher threads while markBlockAsDeleted()
// runs on the thread removing a block. ThreadSanitizer reports the unsynchronized accesses when run against
// the old code.
TEST_F(KnownBlockVersionsTest, concurrentAccess_markAsDeletedWhileUpdatingVersions) {
    constexpr uint64_t NUM_UPDATED_BLOCKS = 100;
    constexpr uint64_t NUM_ITERATIONS = 2000; // enough new entries to make _lastUpdateClientId rehash several times
    constexpr uint64_t NUM_ROUNDS = NUM_ITERATIONS / NUM_UPDATED_BLOCKS;

    std::atomic<uint64_t> rejectedUpdates(0);
    std::thread updater([&] () {
        for (uint64_t i = 0; i < NUM_ITERATIONS; ++i) {
            const BlockId blockId = blockIdForIndex(i % NUM_UPDATED_BLOCKS);
            testobj.incrementVersion(blockId);
            if (!testobj.checkAndUpdateVersion(clientId, blockId, i / NUM_UPDATED_BLOCKS + 1)) {
                ++rejectedUpdates;
            }
        }
    });

    std::atomic<uint64_t> deletedBlocksReportedAsExisting(0);
    std::thread deleter([&] () {
        for (uint64_t i = 0; i < NUM_ITERATIONS; ++i) {
            // A new block id on each iteration, so _lastUpdateClientId keeps growing and rehashing
            const BlockId blockId = blockIdForIndex(NUM_UPDATED_BLOCKS + i);
            testobj.markBlockAsDeleted(blockId);
            if (testobj.blockShouldExist(blockId)) {
                ++deletedBlocksReportedAsExisting;
            }
            if (testobj.existingBlocks().count(blockId) != 0) {
                ++deletedBlocksReportedAsExisting;
            }
        }
    });

    updater.join();
    deleter.join();

    EXPECT_EQ(0u, rejectedUpdates.load());
    EXPECT_EQ(0u, deletedBlocksReportedAsExisting.load());
    for (uint64_t i = 0; i < NUM_UPDATED_BLOCKS; ++i) {
        const BlockId blockId = blockIdForIndex(i);
        EXPECT_TRUE(testobj.blockShouldExist(blockId));
        EXPECT_EQ(NUM_ROUNDS, testobj.getBlockVersion(myClientId, blockId));
        EXPECT_EQ(NUM_ROUNDS, testobj.getBlockVersion(clientId, blockId));
    }
    EXPECT_EQ(NUM_UPDATED_BLOCKS, testobj.existingBlocks().size());
}

// Regression test for the same data race on the other member guarded by the mutex.
// setIntegrityViolationOnPreviousRun() and integrityViolationOnPreviousRun() used to touch
// _integrityViolationOnPreviousRun without taking the mutex, while _loadStateFile()/_saveStateFile()
// access it under the mutex. IntegrityBlockStore2::integrityViolationDetected() calls the setter and
// is reachable from load()/forEachBlock() on any thread, so two threads detecting a violation used to
// write the same bool without synchronization.
TEST_F(KnownBlockVersionsTest, concurrentAccess_integrityViolationOnPreviousRun) {
    constexpr uint64_t NUM_ITERATIONS = 2000;

    std::atomic<uint64_t> numNotSet(0);
    const auto hammer = [&] () {
        for (uint64_t i = 0; i < NUM_ITERATIONS; ++i) {
            testobj.setIntegrityViolationOnPreviousRun(true);
            if (!testobj.integrityViolationOnPreviousRun()) {
                ++numNotSet;
            }
        }
    };
    std::thread thread1(hammer);
    std::thread thread2(hammer);
    thread1.join();
    thread2.join();

    EXPECT_EQ(0u, numNotSet.load());
    EXPECT_TRUE(testobj.integrityViolationOnPreviousRun());
}
