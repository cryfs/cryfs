#include "../../../constexpr/impl/digit_parser.h"

using namespace cpputils;

static_assert(digit_parser::isDigit('0'), "'0' should be recognized as a digit");
static_assert(digit_parser::isDigit('1'), "'1' should be recognized as a digit");
static_assert(digit_parser::isDigit('2'), "'2' should be recognized as a digit");
static_assert(digit_parser::isDigit('3'), "'3' should be recognized as a digit");
static_assert(digit_parser::isDigit('4'), "'4' should be recognized as a digit");
static_assert(digit_parser::isDigit('5'), "'5' should be recognized as a digit");
static_assert(digit_parser::isDigit('6'), "'6' should be recognized as a digit");
static_assert(digit_parser::isDigit('7'), "'7' should be recognized as a digit");
static_assert(digit_parser::isDigit('8'), "'8' should be recognized as a digit");
static_assert(digit_parser::isDigit('9'), "'9' should be recognized as a digit");
static_assert(!digit_parser::isDigit('a'), "'a' should not be recognized as a digit");
static_assert(!digit_parser::isDigit('.'), "'.' should not be recognized as a digit");
static_assert(!digit_parser::isDigit('-'), "'-' should not be recognized as a digit");
static_assert(!digit_parser::isDigit('/'), "'/' should not be recognized as a digit");
static_assert(!digit_parser::isDigit(' '), "' ' should not be recognized as a digit");

static_assert(0 == digit_parser::parseDigit('0'), "'0' should be correctly parsed");
static_assert(1 == digit_parser::parseDigit('1'), "'1' should be correctly parsed");
static_assert(2 == digit_parser::parseDigit('2'), "'2' should be correctly parsed");
static_assert(3 == digit_parser::parseDigit('3'), "'3' should be correctly parsed");
static_assert(4 == digit_parser::parseDigit('4'), "'4' should be correctly parsed");
static_assert(5 == digit_parser::parseDigit('5'), "'5' should be correctly parsed");
static_assert(6 == digit_parser::parseDigit('6'), "'6' should be correctly parsed");
static_assert(7 == digit_parser::parseDigit('7'), "'7' should be correctly parsed");
static_assert(8 == digit_parser::parseDigit('8'), "'8' should be correctly parsed");
static_assert(9 == digit_parser::parseDigit('9'), "'9' should be correctly parsed");
