#include "../../src/version/VersionParser.h"
#include <cstring>

using namespace version;
using cpputils::const_string;

static_assert(0 == VersionParser::extractMajor("0.8"), "\"0.8\" has major version 0");
static_assert(0 == VersionParser::extractMajor("0.8alpha"), "\"0.8alpha\" has major version 0");
static_assert(1 == VersionParser::extractMajor("1.0"), "\"1.0\" has major version 1");
static_assert(1 == VersionParser::extractMajor("1.0alpha"), "\"1.0alpha\" has major version 1");
static_assert(1 == VersionParser::extractMajor("01.0"), "\"01.0\" has major version 1");
static_assert(12 == VersionParser::extractMajor("12.3"), "\"12.3\" has major version 12");
static_assert(12 == VersionParser::extractMajor("12.3alpha"), "\"12.3alpha\" has major version 12");

static_assert(0 == VersionParser::extractMinor("0.0"), "\"0.0\" has minor version 0");
static_assert(1 == VersionParser::extractMinor("0.01"), "\"0.01\" has minor version 1");
static_assert(34 == VersionParser::extractMinor("12.34"), "\"12.34\" has minor version 34");
static_assert(34 == VersionParser::extractMinor("12.34alpha"), "\"12.34alpha\" has minor version 34");

static_assert(const_string("") == VersionParser::extractTag("0.0"), "\"0.0\" has no version tag");
static_assert(const_string("") == VersionParser::extractTag("0.01"), "\"0.01\" has no version tag");
static_assert(const_string("") == VersionParser::extractTag("12.34"), "\"12.34\" has no version tag");
static_assert(const_string("alpha") == VersionParser::extractTag("12.34alpha"), "\"12.34alpha\" has alpha version tag");
static_assert(const_string("rc1") == VersionParser::extractTag("12.34rc1"), "\"12.34rc1\" has rc1 version tag");
static_assert(const_string("rc1") == VersionParser::extractTag("1.0rc1"), "\"1.0rc1\" has rc1 version tag");

static_assert(VersionTag::ALPHA == VersionParser::parseTag("alpha"), "alpha version tag should be parseable");
static_assert(VersionTag::BETA == VersionParser::parseTag("beta"), "beta version tag should be parseable");
static_assert(VersionTag::RC1 == VersionParser::parseTag("rc1"), "rc1 version tag should be parseable");
static_assert(VersionTag::FINAL == VersionParser::parseTag(""), "final version tag should be parseable");

static_assert(Version(1, 0, VersionTag::ALPHA, 0, "commitid") == VersionParser::parse("1.0alpha", 0, "commitid"), "1.0alpha should parse correctly");
static_assert(Version(12, 34, VersionTag::BETA, 0, "commitid") == VersionParser::parse("12.34beta", 0, "commitid"), "12.34beta should parse correctly");
static_assert(Version(0, 8, VersionTag::RC1, 0, "commitid") == VersionParser::parse("0.8rc1", 0, "commitid"), "0.8rc1 should parse correctly");
static_assert(Version(1, 2, VersionTag::FINAL, 0, "commitid") == VersionParser::parse("1.2", 0, "commitid"), "1.2 should parse correctly");
static_assert(Version(1, 2, VersionTag::FINAL, 0, "commitid") == VersionParser::parse("1.02", 0, "commitid"), "1.02 should parse correctly");
static_assert(Version(1, 20, VersionTag::FINAL, 0, "commitid") == VersionParser::parse("1.20", 0, "commitid"), "1.20 should parse correctly");
static_assert(Version(1, 20, VersionTag::FINAL, 0, "commitid") == VersionParser::parse("1.020", 0, "commitid"), "1.020 should parse correctly");

static_assert(Version(1, 20, VersionTag::FINAL, 103, "commitid") == VersionParser::parse("1.020", 103, "commitid"), "commitsSinceVersion should parse correctly");
static_assert(Version(1, 20, VersionTag::FINAL, 103, "other_commitid") == VersionParser::parse("1.020", 103, "other_commitid"), "commitId should parse correctly");
