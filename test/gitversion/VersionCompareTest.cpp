#include <gtest/gtest.h>
#include <gitversion/VersionCompare.h>

using namespace gitversion;
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
    EXPECT_IS_OLDER_THAN("1", "1.0.1");
    EXPECT_IS_OLDER_THAN("1.0.0", "1.1");
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

TEST_F(VersionCompareTest, VersionTags) {
    EXPECT_IS_OLDER_THAN("0.9.3-alpha", "0.9.3-beta");
    EXPECT_IS_OLDER_THAN("1.0-beta", "1.0-rc1");
    EXPECT_IS_OLDER_THAN("1.0-rc1", "1.0-rc2");
    EXPECT_IS_OLDER_THAN("1.0-rc2", "1.0");
    EXPECT_IS_OLDER_THAN("0.9.5", "0.10-m1");
    EXPECT_IS_OLDER_THAN("0.10-m1", "0.10.0");
    EXPECT_IS_OLDER_THAN("1.0-alpha", "1.0");
    EXPECT_IS_SAME_AGE("0.9.3-alpha", "0.9.3-alpha");
    EXPECT_IS_SAME_AGE("1-beta", "1-beta");
    EXPECT_IS_SAME_AGE("0.9.3-rc1", "0.9.3-rc1");
}

TEST_F(VersionCompareTest, DevVersions) {
    EXPECT_IS_OLDER_THAN("0.8", "0.8.1+1.g1234");
    EXPECT_IS_OLDER_THAN("0.8.1", "0.8.2+1.g1234");
    EXPECT_IS_OLDER_THAN("0.8.1+1.g1234", "0.8.2");
    EXPECT_IS_OLDER_THAN("0.8+1.g1234", "0.8.1");
    EXPECT_IS_OLDER_THAN("0.8+1.g1234", "0.9");
    EXPECT_IS_OLDER_THAN("0.9+1.g1234", "0.9+2.g1234");
    EXPECT_IS_SAME_AGE("0.9.1+1.g1234", "0.9.1+1.g3456");
    EXPECT_IS_SAME_AGE("0.9.1+5.g1234", "0.9.1+5.g2345.dirty");
}

TEST_F(VersionCompareTest, DevVersions_VersionTags) {
    EXPECT_IS_OLDER_THAN("0.9.3-alpha+3.gabcd", "0.9.3-alpha+4.gabcd");
    EXPECT_IS_OLDER_THAN("0.9.3-alpha+5.gabcd", "0.9.3-beta");
    EXPECT_IS_OLDER_THAN("0.9.3-alpha+5.gabcd", "0.9.3-beta+1.gabcd");
    EXPECT_IS_OLDER_THAN("0.9.3-alpha+5.gabcd", "1+0.gabcd.dirty");
    EXPECT_IS_OLDER_THAN("0.9.3-alpha+5.gabcd", "1");
    EXPECT_IS_SAME_AGE("0.9.3-alpha+3.gabcd", "0.9.3-alpha+3.gabcd");
}
