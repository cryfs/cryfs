#include <gtest/gtest.h>
#include <cryfs-cli/VersionCompare.h>

using namespace cryfs;
using std::string;

class VersionCompareTest : public ::testing::Test {
public:
    void EXPECT_IS_OLDER_THAN(const string &v1, const string &v2) {
        EXPECT_TRUE(VersionCompare::isOlderThan(v1, v2));
        EXPECT_FALSE(VersionCompare::isOlderThan(v2, v1));
    }

    void EXPECT_IS_SAME_AGE(const string &v1, const string &v2) {
        EXPECT_FALSE(VersionCompare::isOlderThan(v1, v2));
        EXPECT_FALSE(VersionCompare::isOlderThan(v2, v1));
    }
};

TEST_F(VersionCompareTest, IsDifferentVersion) {
    EXPECT_IS_OLDER_THAN("0.8", "0.8.1");
    EXPECT_IS_OLDER_THAN("0.8", "1.0");
    EXPECT_IS_OLDER_THAN("0.8", "1.0.1");
    EXPECT_IS_OLDER_THAN("0.8.1", "1.0");
    EXPECT_IS_OLDER_THAN("0.7.9", "0.8.0");
    EXPECT_IS_OLDER_THAN("1.0.0", "1.0.1");
    EXPECT_IS_OLDER_THAN("1.0.0.0", "1.0.0.1");
    EXPECT_IS_OLDER_THAN("1", "1.0.0.1");
    EXPECT_IS_OLDER_THAN("1.0.0.0", "1.1");
}

TEST_F(VersionCompareTest, IsSameVersion) {
    EXPECT_IS_SAME_AGE("0.8", "0.8");
    EXPECT_IS_SAME_AGE("1.0", "1.0");
    EXPECT_IS_SAME_AGE("1", "1.0");
    EXPECT_IS_SAME_AGE("1.0.0", "1.0.0");
    EXPECT_IS_SAME_AGE("0.8", "0.8.0");
    EXPECT_IS_SAME_AGE("1", "1.0.0.0");
}

TEST_F(VersionCompareTest, ZeroPrefix) {
    EXPECT_IS_OLDER_THAN("1.00.0", "1.0.01");
    EXPECT_IS_SAME_AGE("1.0.01", "1.0.1");
    EXPECT_IS_SAME_AGE("01.0.01", "1.0.1");
}

TEST_F(VersionCompareTest, DevVersions) {
    EXPECT_IS_OLDER_THAN("0.8", "0.8.1.dev1");
    EXPECT_IS_OLDER_THAN("0.8.1", "0.8.2.dev2");
    EXPECT_IS_OLDER_THAN("0.8.1.dev1", "0.8.2");
    EXPECT_IS_OLDER_THAN("0.8.dev1", "0.8.1");
    EXPECT_IS_OLDER_THAN("0.8.dev1", "0.9");
    EXPECT_IS_SAME_AGE("0.9.1.dev5", "0.9.1");
}