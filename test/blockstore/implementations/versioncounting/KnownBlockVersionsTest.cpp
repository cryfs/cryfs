#include <gtest/gtest.h>
#include <blockstore/implementations/versioncounting/KnownBlockVersions.h>

using blockstore::versioncounting::KnownBlockVersions;

class KnownBlockVersionsTest : public ::testing::Test {
public:
    blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");

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
