#include "../../src/version/Version.h"

#include <cstring>
#include <google/gtest/gtest.h>

using namespace version;
using cpputils::const_string;

static_assert(const_string("alpha") == VersionTagToString(VersionTag::ALPHA), "VersionTag::ALPHA toString");
static_assert(const_string("beta") == VersionTagToString(VersionTag::BETA), "VersionTag::BETA toString");
static_assert(const_string("rc1") == VersionTagToString(VersionTag::RC1), "VersionTag::RC1 toString");
static_assert(const_string("") == VersionTagToString(VersionTag::FINAL), "VersionTag::FINAL toString");

static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == Version(1, 0, VersionTag::ALPHA, 0, "commitid"), "Equality for equals");
static_assert(Version(0, 8, VersionTag::FINAL, 2, "commitid") == Version(0, 8, VersionTag::FINAL, 2, "commitid"), "Equality for equals");
static_assert(!(Version(1, 0, VersionTag::ALPHA, 0, "commitid") != Version(1, 0, VersionTag::ALPHA, 0, "commitid")), "Inequality for equals");
static_assert(!(Version(0, 8, VersionTag::FINAL, 2, "commitid") != Version(0, 8, VersionTag::FINAL, 2, "commitid")), "Inequality for equals");

static_assert(!(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == Version(2, 0, VersionTag::ALPHA, 0, "commitid")), "Equality for inequal major");
static_assert(!(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == Version(1, 1, VersionTag::ALPHA, 0, "commitid")), "Equality for inequal minor");
static_assert(!(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == Version(1, 0, VersionTag::FINAL, 0, "commitid")), "Equality for inequal tag");
static_assert(!(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == Version(1, 0, VersionTag::FINAL, 1, "commitid")), "Equality for inequal commitsSinceVersion");
static_assert(!(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == Version(1, 0, VersionTag::FINAL, 0, "commitid2")), "Equality for inequal gitCommitId");
static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") != Version(2, 0, VersionTag::ALPHA, 0, "commitid"), "Inequality for inequal major");
static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") != Version(1, 1, VersionTag::ALPHA, 0, "commitid"), "Inequality for inequal minor");
static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") != Version(1, 0, VersionTag::FINAL, 0, "commitid"), "Inequality for inequal tag");
static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") != Version(1, 0, VersionTag::FINAL, 1, "commitid"), "Inequality for inequal commitsSinceVersion");
static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") != Version(1, 0, VersionTag::FINAL, 0, "commitid2"), "Inequality for inequal gitCommitId");

static_assert(!Version(1, 0, VersionTag::ALPHA, 0, "commitid").is_stable(), "Alpha is not stable");
static_assert(!Version(1, 0, VersionTag::BETA, 0, "commitid").is_stable(), "Beta is not stable");
static_assert(!Version(1, 0, VersionTag::RC1, 0, "commitid").is_stable(), "RC1 is not stable");
static_assert(Version(1, 0, VersionTag::FINAL, 0, "commitid").is_stable(), "Final is stable");
static_assert(!Version(1, 0, VersionTag::FINAL, 1, "commitid").is_stable(), "Final is not stable if there have been commits since");

static_assert(!Version(1, 0, VersionTag::FINAL, 0, "commitid").is_dev(), "Is not dev version when there haven't been commits since the last tag");
static_assert(!Version(1, 0, VersionTag::ALPHA, 0, "commitid").is_dev(), "Is not dev version when there haven't been commits since the last tag, also for alpha versions");
static_assert(Version(1, 0, VersionTag::ALPHA, 1, "commitid").is_dev(), "Is dev version when there haven't been commits since the last tag");
static_assert(Version(1, 0, VersionTag::FINAL, 1, "commitid").is_dev(), "Is dev version when there haven't been commits since the last tag, also for final versions");
static_assert(Version(1, 0, VersionTag::ALPHA, 103, "commitid").is_dev(), "Is dev version when there haven't been commits since the last tag, also for higher commit counts");

TEST(VersionTest, ToString) {
    EXPECT_EQ("0.8alpha", Version(0, 8, VersionTag::ALPHA, 0, "commitid").toString());
    EXPECT_EQ("1.2beta", Version(1, 2, VersionTag::BETA, 0, "commitid").toString());
    EXPECT_EQ("12.0rc1", Version(12, 0, VersionTag::RC1, 0, "commitid").toString());
    EXPECT_EQ("12.34", Version(12, 34, VersionTag::FINAL, 0, "commitid").toString());
}

TEST(VersionTest, ToString_WithCommitsSinceVersion) {
    EXPECT_EQ("0.8alpha-dev2-commitid1", Version(0, 8, VersionTag::ALPHA, 2, "commitid1").toString());
    EXPECT_EQ("1.2beta-dev1-commitid2", Version(1, 2, VersionTag::BETA, 1, "commitid2").toString());
    EXPECT_EQ("12.0rc1-dev5-commitid3", Version(12, 0, VersionTag::RC1, 5, "commitid3").toString());
    EXPECT_EQ("12.34-dev103-commitid4", Version(12, 34, VersionTag::FINAL, 103, "commitid4").toString());
}
