#include <cpp-utils/either.h>
#include <gtest/gtest.h>

using std::string;
using cpputils::either;
using cpputils::make_left;
using cpputils::make_right;

namespace {
class MovableOnly final {
public:
    explicit MovableOnly(int value): _value(value) {}
    MovableOnly(const MovableOnly&) = delete;
    MovableOnly(MovableOnly&&) = default;
    MovableOnly& operator=(const MovableOnly&) = delete;
    MovableOnly& operator=(MovableOnly&&) = default;

    int value() {
        return _value;
    }

private:
    int _value;
};
}

TEST(EitherTest, givenLeft_thenIsLeft) {
    either<int, string> a(4);
    EXPECT_TRUE(a.is_left());
}

TEST(EitherTest, givenLeft_thenIsNotRight) {
    either<int, string> a(4);
    EXPECT_FALSE(a.is_right());
}

TEST(EitherTest, givenLeft_whenQueryingLeft_thenValueIsCorrect) {
    either<int, string> a(4);
    EXPECT_EQ(4, a.left());
}

TEST(EitherTest, givenLeft_whenQueryingRight_thenThrows) {
    either<int, string> a(4);
    EXPECT_ANY_THROW(a.right());
}

TEST(EitherTest, givenLeft_whenQueryingOptLeft_thenValueIsCorrect) {
    either<int, string> a(4);
    EXPECT_EQ(4, a.left_opt().value());
}

TEST(EitherTest, givenLeft_whenQueryingOptRight_thenIsNone) {
    either<int, string> a(4);
    EXPECT_EQ(boost::none, a.right_opt());
}

TEST(EitherTest, givenRight_thenIsRight) {
    either<int, string> a("4");
    EXPECT_TRUE(a.is_right());
}

TEST(EitherTest, givenRight_thenIsNotLeft) {
    either<int, string> a("4");
    EXPECT_FALSE(a.is_left());
}

TEST(EitherTest, givenRight_whenQueryingRight_thenValueIsCorrect) {
    either<int, string> a("4");
    EXPECT_EQ("4", a.right());
}

TEST(EitherTest, givenRight_whenQueryingLeft_thenThrows) {
    either<int, string> a("4");
    EXPECT_ANY_THROW(a.left());
}

TEST(EitherTest, givenRight_whenQueryingRightOpt_thenValueIsCorrect) {
    either<int, string> a("4");
    EXPECT_EQ("4", a.right_opt().value());
}

TEST(EitherTest, givenRight_whenQueryingLeftOpt_thenThrows) {
    either<int, string> a("4");
    EXPECT_EQ(boost::none, a.left_opt());
}

TEST(EitherTest, whenCopyConstructingLeft_thenValueIsCorrect) {
    string a = "4";
    either<string, int> b(a);
    EXPECT_EQ(a, b.left());
}

TEST(EitherTest, whenMoveConstructingLeft_thenValueIsCorrect) {
    string a = "4";
    either<string, int> b(std::move(a));
    EXPECT_EQ("4", b.left());
}

TEST(EitherTest, whenCopyConstructingRight_thenValueIsCorrect) {
    string a = "4";
    either<int, string> b(a);
    EXPECT_EQ(a, b.right());
}

TEST(EitherTest, whenMoveConstructingRight_thenValueIsCorrect) {
    string a = "4";
    either<int, string> b(std::move(a));
    EXPECT_EQ("4", b.right());
}

//TEST(EitherTest, whenMakingLeft_thenIsLeft) {
//    auto a = make_left<int, string>(4);
//    EXPECT_TRUE(a.is_left());
//}
//
//TEST(EitherTest, whenMakingRight_thenIsRight) {
//    auto a = make_right<int, string>("4");
//    EXPECT_TRUE(a.is_right());
//}


// TODO Test MovableOnly content type
// TODO Test Left == Right (Same type)
// TODO Test operator== and !=
// TODO Test copy/move constructor
// TODO Test destruction
// TODO Test make_left / make_right
// TODO Test noexcept tags are correct
