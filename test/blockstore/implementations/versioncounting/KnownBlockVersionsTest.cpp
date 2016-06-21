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

    TempFile stateFile;
    KnownBlockVersions testobj;
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

TEST_F(KnownBlockVersionsTest, checkAndUpdate_newEntry_zero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 0));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_newEntry_nonzero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_equal_zero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 0));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 0));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_equal_nonzero) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_nonequal) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 101));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_invalid) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(key, 99));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_existingEntry_invalidDoesntModifyEntry) {
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(key, 99));

    EXPECT_FALSE(testobj.checkAndUpdateVersion(key, 99));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 100));
}

TEST_F(KnownBlockVersionsTest, checkAndUpdate_twoEntriesDontInfluenceEachOther) {
    testobj.updateVersion(key, 100);
    testobj.updateVersion(key2, 100);

    testobj.updateVersion(key, 150);

    EXPECT_FALSE(testobj.checkAndUpdateVersion(key, 149));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key, 150));
    EXPECT_FALSE(testobj.checkAndUpdateVersion(key2, 99));
    EXPECT_TRUE(testobj.checkAndUpdateVersion(key2, 100));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_empty) {
    TempFile stateFile(false);
    KnownBlockVersions(stateFile.path());

    EXPECT_TRUE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(key, 0));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_oneentry) {
    TempFile stateFile(false);
    KnownBlockVersions(stateFile.path()).updateVersion(key, 100);

    EXPECT_FALSE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(key, 99));
    EXPECT_TRUE(KnownBlockVersions(stateFile.path()).checkAndUpdateVersion(key, 100));
}

TEST_F(KnownBlockVersionsTest, saveAndLoad_twoentries) {
    TempFile stateFile(false);
    {
        KnownBlockVersions obj(stateFile.path());
        obj.updateVersion(key, 100);
        obj.updateVersion(key2, 50);
    }

    KnownBlockVersions obj(stateFile.path());
    EXPECT_FALSE(obj.checkAndUpdateVersion(key, 99));
    EXPECT_TRUE(obj.checkAndUpdateVersion(key, 100));
    EXPECT_FALSE(obj.checkAndUpdateVersion(key2, 49));
    EXPECT_TRUE(obj.checkAndUpdateVersion(key2, 50));
}
