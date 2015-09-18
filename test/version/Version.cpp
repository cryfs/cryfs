#include "../../src/version/Version.h"

#include <cstring>
#include <google/gtest/gtest.h>

using namespace version;
using cpputils::const_string;

static_assert(const_string("alpha") == VersionTagToString(VersionTag::ALPHA), "VersionTag::ALPHA toString");
static_assert(const_string("beta") == VersionTagToString(VersionTag::BETA), "VersionTag::BETA toString");
static_assert(const_string("rc1") == VersionTagToString(VersionTag::RC1), "VersionTag::RC1 toString");
static_assert(const_string("") == VersionTagToString(VersionTag::FINAL), "VersionTag::FINAL toString");

static_assert(Version(1, 0, VersionTag::ALPHA) == Version(1, 0, VersionTag::ALPHA), "Equality for equals");
static_assert(!(Version(1, 0, VersionTag::ALPHA) != Version(1, 0, VersionTag::ALPHA)), "Inequality for equals");

static_assert(!(Version(1, 0, VersionTag::ALPHA) == Version(2, 0, VersionTag::ALPHA)), "Equality for inequal major");
static_assert(!(Version(1, 0, VersionTag::ALPHA) == Version(1, 1, VersionTag::ALPHA)), "Equality for inequal minor");
static_assert(!(Version(1, 0, VersionTag::ALPHA) == Version(1, 0, VersionTag::FINAL)), "Equality for inequal tag");
static_assert(Version(1, 0, VersionTag::ALPHA) != Version(2, 0, VersionTag::ALPHA), "Inequality for inequal major");
static_assert(Version(1, 0, VersionTag::ALPHA) != Version(1, 1, VersionTag::ALPHA), "Inequality for inequal minor");
static_assert(Version(1, 0, VersionTag::ALPHA) != Version(1, 0, VersionTag::FINAL), "Inequality for inequal tag");

static_assert(!Version(1, 0, VersionTag::ALPHA).is_stable(), "Alpha is not stable");
static_assert(!Version(1, 0, VersionTag::BETA).is_stable(), "Beta is not stable");
static_assert(!Version(1, 0, VersionTag::RC1).is_stable(), "RC1 is not stable");
static_assert(Version(1, 0, VersionTag::FINAL).is_stable(), "Final is stable");

TEST(VersionTest, ToString) {
    EXPECT_EQ("0.8alpha", Version(0, 8, VersionTag::ALPHA).toString());
    EXPECT_EQ("1.2beta", Version(1, 2, VersionTag::BETA).toString());
    EXPECT_EQ("12.0rc1", Version(12, 0, VersionTag::RC1).toString());
    EXPECT_EQ("12.34", Version(12, 34, VersionTag::FINAL).toString());
}
