#include "../../src/version/VersionParser.h"
#include <cstring>

using namespace version;

static_assert(VersionParser::isDigit('0'), "'0' should be recognized as a digit");
static_assert(VersionParser::isDigit('1'), "'1' should be recognized as a digit");
static_assert(VersionParser::isDigit('2'), "'2' should be recognized as a digit");
static_assert(VersionParser::isDigit('3'), "'3' should be recognized as a digit");
static_assert(VersionParser::isDigit('4'), "'4' should be recognized as a digit");
static_assert(VersionParser::isDigit('5'), "'5' should be recognized as a digit");
static_assert(VersionParser::isDigit('6'), "'6' should be recognized as a digit");
static_assert(VersionParser::isDigit('7'), "'7' should be recognized as a digit");
static_assert(VersionParser::isDigit('8'), "'8' should be recognized as a digit");
static_assert(VersionParser::isDigit('9'), "'9' should be recognized as a digit");
static_assert(!VersionParser::isDigit('a'), "'a' should not be recognized as a digit");
static_assert(!VersionParser::isDigit('.'), "'.' should not be recognized as a digit");
static_assert(!VersionParser::isDigit('-'), "'-' should not be recognized as a digit");
static_assert(!VersionParser::isDigit('/'), "'/' should not be recognized as a digit");
static_assert(!VersionParser::isDigit(' '), "' ' should not be recognized as a digit");

static_assert(0 == VersionParser::parseDigit('0'), "'0' should be correctly parsed");
static_assert(1 == VersionParser::parseDigit('1'), "'1' should be correctly parsed");
static_assert(2 == VersionParser::parseDigit('2'), "'2' should be correctly parsed");
static_assert(3 == VersionParser::parseDigit('3'), "'3' should be correctly parsed");
static_assert(4 == VersionParser::parseDigit('4'), "'4' should be correctly parsed");
static_assert(5 == VersionParser::parseDigit('5'), "'5' should be correctly parsed");
static_assert(6 == VersionParser::parseDigit('6'), "'6' should be correctly parsed");
static_assert(7 == VersionParser::parseDigit('7'), "'7' should be correctly parsed");
static_assert(8 == VersionParser::parseDigit('8'), "'8' should be correctly parsed");
static_assert(9 == VersionParser::parseDigit('9'), "'9' should be correctly parsed");

static_assert(0 == VersionParser::numDigits("ab"), "\"ab\" doesn't have any digits");
static_assert(0 == VersionParser::numDigits(""), "\"\" doesn't have any digits");
static_assert(1 == VersionParser::numDigits("5"), "\"5\" has 1 digit");
static_assert(1 == VersionParser::numDigits("5a"), "\"5\" has 1 digit");
static_assert(10 == VersionParser::numDigits("5594839203a"), "\"5594839203a\" has 1 digit");
static_assert(10 == VersionParser::numDigits("5594839203"), "\"5594839203\" has 1 digit");

static_assert(0 == VersionParser::parseNumber("0"), "\"0\" should be correctly parsed");
static_assert(0 == VersionParser::parseNumber("0a"), "\"0a\" should be correctly parsed");
static_assert(0 == VersionParser::parseNumber("0."), "\"0.\" should be correctly parsed");
static_assert(3 == VersionParser::parseNumber("3"), "\"3\" should be correctly parsed");
static_assert(12 == VersionParser::parseNumber("12"), "\"12\" should be correctly parsed");
static_assert(123 == VersionParser::parseNumber("123"), "\"123\" should be correctly parsed");
static_assert(123 == VersionParser::parseNumber("0123"), "\"0123\" should be correctly parsed");
static_assert(1 == VersionParser::parseNumber("001a"), "\"001a\" should be correctly parsed");
static_assert(1230 == VersionParser::parseNumber("1230"), "\"1230\" should be correctly parsed");
static_assert(1230 == VersionParser::parseNumber("1230beta"), "\"1230beta\" should be correctly parsed");
static_assert(357532 == VersionParser::parseNumber("357532"), "\"357532\" should be correctly parsed");
static_assert(357532 == VersionParser::parseNumber("357532alpha"), "\"357532alpha\" should be correctly parsed");
static_assert(357532 == VersionParser::parseNumber("357532.4"), "\"357532.4\" should be correctly parsed");

static_assert(0 == strcmp("bla", VersionParser::skipNumber("bla")), "\"bla\" has no number to skip");
static_assert(0 == strcmp("alpha", VersionParser::skipNumber("0alpha")), "\"0alpha\" should skip the 0");
static_assert(0 == strcmp(".3alpha", VersionParser::skipNumber("12.3alpha")), "\"12.3alpha\" should skip the 12");

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

static_assert(0 == strlen(VersionParser::extractTag("0.0")), "\"0.0\" has no version tag");
static_assert(0 == strlen(VersionParser::extractTag("0.01")), "\"0.01\" has no version tag");
static_assert(0 == strlen(VersionParser::extractTag("12.34")), "\"12.34\" has no version tag");
static_assert(0 == strcmp("alpha", VersionParser::extractTag("12.34alpha")), "\"12.34alpha\" has alpha version tag");
static_assert(0 == strcmp("rc1", VersionParser::extractTag("12.34rc1")), "\"12.34rc1\" has rc1 version tag");
static_assert(0 == strcmp("rc1", VersionParser::extractTag("1.0rc1")), "\"1.0rc1\" has rc1 version tag");

static_assert(VersionTag::ALPHA == VersionParser::parseTag("alpha"), "alpha version tag should be parseable");
static_assert(VersionTag::BETA == VersionParser::parseTag("beta"), "beta version tag should be parseable");
static_assert(VersionTag::RC1 == VersionParser::parseTag("rc1"), "rc1 version tag should be parseable");
static_assert(VersionTag::FINAL == VersionParser::parseTag(""), "final version tag should be parseable");

static_assert(Version(1, 0, VersionTag::ALPHA) == VersionParser::parse("1.0alpha"), "1.0alpha should parse correctly");
static_assert(Version(12, 34, VersionTag::BETA) == VersionParser::parse("12.34beta"), "12.34beta should parse correctly");
static_assert(Version(0, 8, VersionTag::RC1) == VersionParser::parse("0.8rc1"), "0.8rc1 should parse correctly");
static_assert(Version(1, 2, VersionTag::FINAL) == VersionParser::parse("1.2"), "1.2 should parse correctly");
static_assert(Version(1, 2, VersionTag::FINAL) == VersionParser::parse("1.02"), "1.02 should parse correctly");
static_assert(Version(1, 20, VersionTag::FINAL) == VersionParser::parse("1.20"), "1.20 should parse correctly");
static_assert(Version(1, 20, VersionTag::FINAL) == VersionParser::parse("1.020"), "1.020 should parse correctly");
