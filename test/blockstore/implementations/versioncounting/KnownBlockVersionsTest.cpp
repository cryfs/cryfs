#include <gtest/gtest.h>
#include <blockstore/implementations/versioncounting/KnownBlockVersions.h>
#include <cpp-utils/tempfile/TempFile.h>

using blockstore::versioncounting::KnownBlockVersions;
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
        EXPECT_TRUE(testobj->checkAndUpdateVersion(clientId, key, version));
    }
};

TEST_F(KnownBlockVersionsTest, update_newEntry_zero) {
    testobj.updateVersion(key, 0);
}

TEST_F(KnownBlockVersionsTest, update_newEntry_nonzero) {
    testobj.updateVersion(key, 100);
}

TEST_F(KnownBlockVersionsTest, update_existingEntry_equal_zero) {
    testobj.updateVersion(key, 0);
    testobj.updateVersion(key, 0);
}

TEST_F(KnownBlockVersionsTest, update_existingEntry_equal_nonzero) {
    testobj.updateVersion(key, 100);
    testobj.updateVersion(key, 100);
}

TEST_F(KnownBlockVersionsTest, update_existingEntry_nonequal) {
    testobj.updateVersion(key, 100);
    testobj.updateVersion(key, 101);
}

TEST_F(KnownBlockVersionsTest, update_existingEntry_invalid) {
    testobj.updateVersion(key, 100);
    EXPECT_ANY_THROW(
            testobj.updateVersion(key, 99);
    );
}

TEST_F(KnownBlockVersionsTest, update_updatesOwnClientId) {
    testobj.updateVersion(key, 100);
    EXPECT_VERSION_IS(100, &testobj, key, testobj.myClientId());
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_newEntry_zero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 0));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_newEntry_nonzero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_equal_zero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 0));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 0));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_equal_nonzero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_nonequal) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 101));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_invalid) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 99));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_invalidDoesntModifyEntry) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(clientId, key, 100));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(clientId, key, 99));

    EXPECT_VERSION_IS(100, &testobj, key, clientId);
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

TEST_F(KnownBlockVersionsTest, saveAndLoad_empty) {
    TempFile stateFile(false);
    KnownBlockVersions(stateFile.path());

    EXPECT_TRUE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(clientId, key, 0));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_oneentry) {
    TempFile stateFile(false);
    EXPECT_TRUE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(clientId, key, 100));

    KnownBlockVersions obj(stateFile.path());
    EXPECT_VERSION_IS(100, &obj, key, clientId);
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_threeentries) {
    TempFile stateFile(false);
    {
        KnownBlockVersions obj(stateFile.path());
        obj.updateVersion(key, 100);
        obj.updateVersion(key2, 50);
        EXPECT_TRUE(obj.checkAndUpdateVersion(clientId, key, 150));
    }

    KnownBlockVersions obj(stateFile.path());
    EXPECT_VERSION_IS(100, &obj, key, obj.myClientId());
    EXPECT_VERSION_IS(50, &obj, key2, obj.myClientId());
    EXPECT_VERSION_IS(150, &obj, key, clientId);
}
