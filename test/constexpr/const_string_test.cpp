#include "../../constexpr/const_string.h"
#include <google/gtest/gtest.h>

using namespace cpputils;
using std::ostringstream;
using std::string;

// ----------------------------------------------
// size()
// ----------------------------------------------
static_assert(6 == const_string("Hello ").size(), "Size of \"Hello \" is 6");
static_assert(1 == const_string(" ").size(), "Size of \" \" is 6");
static_assert(0 == const_string("").size(), "Size of \"\" is 6");


// ----------------------------------------------
// operator[]
// ----------------------------------------------
static_assert('a' == const_string("a")[0], "\"a\"[0] == 'a'");
static_assert('a' == const_string("abc")[0], "\"abc\"[0] == 'a'");
static_assert('b' == const_string("abc")[1], "\"abc\"[1] == 'b'");
static_assert('c' == const_string("abc")[2], "\"abc\"[1] == 'c'");

static_assert('c' == const_string("abc").dropPrefix(1)[1], "operator[] is not broken after calling dropPrefix()");

// ----------------------------------------------
// operator== and operator!=
// ----------------------------------------------
static_assert(const_string("") == const_string(""), "\"\" should be equal to \"\"");
static_assert(const_string("a") == const_string("a"), "\"a\" should be equal to \"a\"");
static_assert(const_string("ab") == const_string("ab"), "\"ab\" should be equal to \"ab\"");
static_assert(!(const_string("") != const_string("")), "\"\" should not be different to \"\"");
static_assert(!(const_string("a") != const_string("a")), "\"a\" should not be different to \"a\"");
static_assert(!(const_string("ab") != const_string("ab")), "\"ab\" should not be different to \"ab\"");

static_assert(!(const_string("a") == const_string("A")), "\"a\" should not be equal to \"A\"");
static_assert(!(const_string("ab") == const_string("abc")), "\"ab\" should not be equal to \"abc\"");
static_assert(!(const_string("abc") == const_string("ab")), "\"abc\" should not be equal to \"ab\"");
static_assert(!(const_string("a") == const_string("")), "\"a\" should not be equal to \"\"");
static_assert(!(const_string("") == const_string("a")), "\"\" should not be equal to \"a\"");
static_assert(const_string("a") != const_string("A"), "\"a\" should be different to \"A\"");
static_assert(const_string("ab") != const_string("abc"), "\"ab\" should be different to \"abc\"");
static_assert(const_string("abc") != const_string("ab"), "\"abc\" should be different to \"ab\"");
static_assert(const_string("a") != const_string(""), "\"a\" should be different to \"\"");
static_assert(const_string("") != const_string("a"), "\"\" should be different to \"a\"");



// ----------------------------------------------
// dropPrefix(), dropSuffix() and substr()
// ----------------------------------------------
static_assert(const_string("bc") == const_string("abc").dropPrefix(1),
              "Dropping the first character of \"abc\" should yield \"bc\"");
static_assert(const_string("c") == const_string("abc").dropPrefix(1).dropPrefix(1),
              "Dropping prefixes should be chainable");
static_assert(const_string("c") == const_string("abc").dropPrefix(2),
              "Dropping the first two characters of \"abc\" should yield \"c\"");
static_assert(const_string("") == const_string("abc").dropPrefix(3),
              "Dropping the first three characters of \"abc\" should yield \"\"");

static_assert(const_string("ab") == const_string("abc").dropSuffix(1),
              "Dropping the last character of \"abc\" should yield \"ab\"");
static_assert(const_string("a") == const_string("abc").dropSuffix(1).dropSuffix(1),
              "Dropping suffixes should be chainable");
static_assert(const_string("a") == const_string("abc").dropSuffix(2),
              "Dropping the last two characters of \"abc\" should yield \"a\"");
static_assert(const_string("") == const_string("abc").dropSuffix(3),
              "Dropping the last three characters of \"abc\" should yield \"\"");

static_assert(const_string("bc") == const_string("abc").substr(1, 2),
              "Dropping the first character of \"abc\" should yield \"bc\"");
static_assert(const_string("ab") == const_string("abc").substr(0, 2),
              "Dropping the last character of \"abc\" should yield \"ab\"");
static_assert(const_string("bc") == const_string("abcd").substr(1, 2),
              "Dropping the first and last character of \"abcd\" should yield \"bc\"");

constexpr const_string val = const_string("abc");
static_assert(val.dropSuffix(1) != val,
              "Even when working with the same underlying object, dropping a suffix makes it a different object.");
static_assert(val.dropPrefix(1) != val,
              "Even when working with the same underlying object, dropping a prefix makes it a different object.");


// ----------------------------------------------
// sizeOfUIntPrefix(), parseUIntPrefix() and dropUIntPrefix()
// ----------------------------------------------

static_assert(0 == const_string("ab").sizeOfUIntPrefix(), "\"ab\" doesn't have any digits");
static_assert(0 == const_string("").sizeOfUIntPrefix(), "\"\" doesn't have any digits");
static_assert(1 == const_string("5").sizeOfUIntPrefix(), "\"5\" has 1 digit");
static_assert(1 == const_string("5a").sizeOfUIntPrefix(), "\"5\" has 1 digit");
static_assert(10 == const_string("5594839203a").sizeOfUIntPrefix(), "\"5594839203a\" has 1 digit");
static_assert(10 == const_string("5594839203").sizeOfUIntPrefix(), "\"5594839203\" has 1 digit");

static_assert(0 == const_string("0").parseUIntPrefix(), "\"0\" should be correctly parsed");
static_assert(0 == const_string("0a").parseUIntPrefix(), "\"0a\" should be correctly parsed");
static_assert(0 == const_string("0.").parseUIntPrefix(), "\"0.\" should be correctly parsed");
static_assert(3 == const_string("3").parseUIntPrefix(), "\"3\" should be correctly parsed");
static_assert(12 == const_string("12").parseUIntPrefix(), "\"12\" should be correctly parsed");
static_assert(123 == const_string("123").parseUIntPrefix(), "\"123\" should be correctly parsed");
static_assert(123 == const_string("0123").parseUIntPrefix(), "\"0123\" should be correctly parsed");
static_assert(1 == const_string("001a").parseUIntPrefix(), "\"001a\" should be correctly parsed");
static_assert(1230 == const_string("1230").parseUIntPrefix(), "\"1230\" should be correctly parsed");
static_assert(1230 == const_string("1230beta").parseUIntPrefix(), "\"1230beta\" should be correctly parsed");
static_assert(357532 == const_string("357532").parseUIntPrefix(), "\"357532\" should be correctly parsed");
static_assert(357532 == const_string("357532alpha").parseUIntPrefix(), "\"357532alpha\" should be correctly parsed");
static_assert(357532 == const_string("357532.4").parseUIntPrefix(), "\"357532.4\" should be correctly parsed");

static_assert(const_string("bla") == const_string("bla").dropUIntPrefix(), "\"bla\" has no number to skip");
static_assert(const_string("alpha") == const_string("0alpha").dropUIntPrefix(), "\"0alpha\" should skip the 0");
static_assert(const_string(".3alpha") == const_string("12.3alpha").dropUIntPrefix(),
              "\"12.3alpha\" should skip the 12");
static_assert(const_string("-5") == const_string("-5").dropUIntPrefix(),
              "\"-5\" is not unsigned and should skip nothing");
static_assert(const_string("") == const_string("").dropUIntPrefix(), "\"\" should skip nothing");



// ----------------------------------------------
// toStdString()
// ----------------------------------------------
TEST(const_string_test, toStdString) {
    EXPECT_EQ("", const_string("").toStdString());
    EXPECT_EQ("a", const_string("a").toStdString());
    EXPECT_EQ("abc", const_string("abc").toStdString());
    EXPECT_EQ("abc", const_string("prefix_abc_suffix").substr(7,3).toStdString());
}

// ----------------------------------------------
// operator<<
// ----------------------------------------------
void EXPECT_OUTPUTS(const string &expected, const const_string &testObj) {
    ostringstream stream;
    stream << testObj;
    EXPECT_EQ(expected, stream.str());
}

TEST(const_string_test, OutputOperator) {
    EXPECT_OUTPUTS("", const_string(""));
    EXPECT_OUTPUTS("a", const_string("a"));
    EXPECT_OUTPUTS("abc", const_string("abc"));
    EXPECT_OUTPUTS("abc", const_string("prefix_abc_suffix").substr(7,3));
}