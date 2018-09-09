#include <cpp-utils/value_type/ValueType.h>
#include <gtest/gtest.h>
#include <utility>
#include <unordered_set>
#include <set>

using cpputils::value_type::IdValueType;
using cpputils::value_type::OrderedIdValueType;
using cpputils::value_type::QuantityValueType;

namespace {

struct MyIdValueType : IdValueType<MyIdValueType, int64_t> {
    constexpr explicit MyIdValueType(int64_t val): IdValueType(val) {}
};

struct MyOrderedIdValueType : OrderedIdValueType<MyOrderedIdValueType, int64_t> {
    constexpr explicit MyOrderedIdValueType(int64_t val): OrderedIdValueType(val) {}
};

struct MyQuantityValueType : QuantityValueType<MyQuantityValueType, int64_t> {
    constexpr explicit MyQuantityValueType(int64_t val): QuantityValueType(val) {}
};

}
DEFINE_HASH_FOR_VALUE_TYPE(MyIdValueType);
DEFINE_HASH_FOR_VALUE_TYPE(MyOrderedIdValueType);
DEFINE_HASH_FOR_VALUE_TYPE(MyQuantityValueType);
namespace {

/**
 * Tests for IdValueType
 */

template<class Type>
struct IdValueTypeTest_constexpr_test {
    static constexpr Type test_constructor = Type(5);
    static_assert(Type(5) == test_constructor, "");

    static constexpr Type test_copy_constructor = test_constructor;
    static_assert(Type(5) == test_copy_constructor, "");

    static constexpr Type test_copy_assignment = (Type(4) = test_copy_constructor);
    static_assert(test_copy_assignment == Type(5), "");

    static constexpr Type test_move_assignment = (Type(4) = Type(3));
    static_assert(test_move_assignment == Type(3), "");

    static_assert(Type(5) == Type(5), "");
    static_assert(Type(5) != Type(6), "");

    static constexpr bool success = true;
};
static_assert(IdValueTypeTest_constexpr_test<MyIdValueType>::success, "");
static_assert(IdValueTypeTest_constexpr_test<MyOrderedIdValueType>::success, "");
static_assert(IdValueTypeTest_constexpr_test<MyQuantityValueType>::success, "");


template<class Type> class IdValueTypeTest : public testing::Test {
};
using IdValueTypeTest_types = testing::Types<MyIdValueType, MyOrderedIdValueType, MyQuantityValueType>;
TYPED_TEST_CASE(IdValueTypeTest, IdValueTypeTest_types);


TYPED_TEST(IdValueTypeTest, Equality) {
    TypeParam obj1(4);
    TypeParam obj2(4);
    TypeParam obj3(5);

    EXPECT_TRUE(obj1 == obj2);
    EXPECT_TRUE(obj2 == obj1);;
    EXPECT_FALSE(obj1 == obj3);
    EXPECT_FALSE(obj3 == obj1);

    EXPECT_FALSE(obj1 != obj2);
    EXPECT_FALSE(obj2 != obj1);
    EXPECT_TRUE(obj1 != obj3);
    EXPECT_TRUE(obj3 != obj1);
}

TYPED_TEST(IdValueTypeTest, Constructor) {
    TypeParam obj(4);
    EXPECT_TRUE(obj == TypeParam(4));
}

TYPED_TEST(IdValueTypeTest, CopyConstructor) {
    TypeParam obj(2);
    TypeParam obj2(obj);
    EXPECT_TRUE(obj2 == TypeParam(2));
    EXPECT_TRUE(obj == obj2);
}

TYPED_TEST(IdValueTypeTest, MoveConstructor) {
    TypeParam obj(2);
    TypeParam obj2(std::move(obj));
    EXPECT_TRUE(obj2 == TypeParam(2));
}

TYPED_TEST(IdValueTypeTest, CopyAssignment) {
    TypeParam obj(3);
    TypeParam obj2(2);
    obj2 = obj;
    EXPECT_TRUE(obj2 == TypeParam(3));
    EXPECT_TRUE(obj == obj2);
}

TYPED_TEST(IdValueTypeTest, CopyAssignment_Return) {
    TypeParam obj(3);
    TypeParam obj2(2);
    EXPECT_TRUE((obj2 = obj) == TypeParam(3));
}

TYPED_TEST(IdValueTypeTest, MoveAssignment) {
    TypeParam obj(3);
    TypeParam obj2(2);
    obj2 = std::move(obj);
    EXPECT_TRUE(obj2 == TypeParam(3));
}

TYPED_TEST(IdValueTypeTest, MoveAssignment_Return) {
    TypeParam obj(3);
    TypeParam obj2(2);
    EXPECT_TRUE((obj2 = std::move(obj)) == TypeParam(3));
}

TYPED_TEST(IdValueTypeTest, Hash) {
    TypeParam obj(3);
    TypeParam obj2(3);
    EXPECT_EQ(std::hash<TypeParam>()(obj), std::hash<TypeParam>()(obj2));
}

TYPED_TEST(IdValueTypeTest, UnorderedSet) {
    std::unordered_set<TypeParam> set;
    set.insert(TypeParam(3));
    EXPECT_EQ(1u, set.count(TypeParam(3)));
}





/**
 * Tests for OrderedIdValueType
 */

template<class Type>
struct OrderedIdValueTypeTest_constexpr_test {
    static_assert(Type(3) < Type(4), "");
    static_assert(!(Type(4) < Type(3)), "");
    static_assert(!(Type(3) < Type(3)), "");

    static_assert(Type(4) > Type(3), "");
    static_assert(!(Type(3) > Type(4)), "");
    static_assert(!(Type(3) > Type(3)), "");

    static_assert(Type(3) <= Type(4), "");
    static_assert(!(Type(4) <= Type(3)), "");
    static_assert(Type(3) <= Type(3), "");

    static_assert(Type(4) >= Type(3), "");
    static_assert(!(Type(3) >= Type(4)), "");
    static_assert(Type(3) >= Type(3), "");

    static constexpr bool success = true;
};
static_assert(OrderedIdValueTypeTest_constexpr_test<MyOrderedIdValueType>::success, "");
static_assert(OrderedIdValueTypeTest_constexpr_test<MyQuantityValueType>::success, "");

template<class Type> class OrderedIdValueTypeTest : public testing::Test {};
using OrderedIdValueTypeTest_types = testing::Types<MyOrderedIdValueType, MyQuantityValueType>;
TYPED_TEST_CASE(OrderedIdValueTypeTest, OrderedIdValueTypeTest_types);

TYPED_TEST(OrderedIdValueTypeTest, LessThan) {
    TypeParam a(3);
    TypeParam b(3);
    TypeParam c(4);
    EXPECT_FALSE(a < a);
    EXPECT_FALSE(a < b);
    EXPECT_TRUE(a < c);
    EXPECT_FALSE(b < a);
    EXPECT_FALSE(b < b);
    EXPECT_TRUE(b < c);
    EXPECT_FALSE(c < a);
    EXPECT_FALSE(c < b);
    EXPECT_FALSE(c < c);
}

TYPED_TEST(OrderedIdValueTypeTest, GreaterThan) {
    TypeParam a(3);
    TypeParam b(3);
    TypeParam c(4);
    EXPECT_FALSE(a > a);
    EXPECT_FALSE(a > b);
    EXPECT_FALSE(a > c);
    EXPECT_FALSE(b > a);
    EXPECT_FALSE(b > b);
    EXPECT_FALSE(b > c);
    EXPECT_TRUE(c > a);
    EXPECT_TRUE(c > b);
    EXPECT_FALSE(c > c);
}

TYPED_TEST(OrderedIdValueTypeTest, LessOrEqualThan) {
    TypeParam a(3);
    TypeParam b(3);
    TypeParam c(4);
    EXPECT_TRUE(a <= a);
    EXPECT_TRUE(a <= b);
    EXPECT_TRUE(a <= c);
    EXPECT_TRUE(b <= a);
    EXPECT_TRUE(b <= b);
    EXPECT_TRUE(b <= c);
    EXPECT_FALSE(c <= a);
    EXPECT_FALSE(c <= b);
    EXPECT_TRUE(c <= c);
}

TYPED_TEST(OrderedIdValueTypeTest, GreaterOrEqualThan) {
    TypeParam a(3);
    TypeParam b(3);
    TypeParam c(4);
    EXPECT_TRUE(a >= a);
    EXPECT_TRUE(a >= b);
    EXPECT_FALSE(a >= c);
    EXPECT_TRUE(b >= a);
    EXPECT_TRUE(b >= b);
    EXPECT_FALSE(b >= c);
    EXPECT_TRUE(c >= a);
    EXPECT_TRUE(c >= b);
    EXPECT_TRUE(c >= c);
}

TYPED_TEST(OrderedIdValueTypeTest, Set) {
    std::set<TypeParam> set;
    set.insert(TypeParam(3));
    EXPECT_EQ(1u, set.count(TypeParam(3)));
}








/**
 * Tests for QuantityValueType
 */

template<class Type>
struct QuantityValueTypeTest_constexpr_test {
    static_assert(++Type(3) == Type(4), "");
    static_assert(Type(3)++ == Type(3), "");
    static_assert(--Type(3) == Type(2), "");
    static_assert(Type(3)-- == Type(3), "");
    static_assert((Type(3) += Type(2)) == Type(5), "");
    static_assert((Type(3) -= Type(2)) == Type(1), "");
    static_assert((Type(3) *= 2) == Type(6), "");
    static_assert((Type(6) /= 2) == Type(3), "");
    static_assert((Type(7) /= 3) == Type(2), "");
    static_assert((Type(7) %= 3) == Type(1), "");
    static_assert(Type(3) + Type(2) == Type(5), "");
    static_assert(Type(3) - Type(2) == Type(1), "");
    static_assert(Type(3) * 2 == Type(6), "");
    static_assert(2 * Type(3) == Type(6), "");
    static_assert(Type(6) / 2 == Type(3), "");
    static_assert(Type(6) / Type(2) == 3, "");
    static_assert(Type(7) / 3 == Type(2), "");
    static_assert(Type(7) / Type(3) == 2, "");
    static_assert(Type(7) % 3 == Type(1), "");
    static_assert(Type(7) % Type(3) == 1, "");

    static constexpr bool success = true;
};
static_assert(QuantityValueTypeTest_constexpr_test<MyQuantityValueType>::success, "");

template<class Type> class QuantityValueTypeTest : public testing::Test {};
using QuantityValueTypeTest_types = testing::Types<MyQuantityValueType>;
TYPED_TEST_CASE(QuantityValueTypeTest, QuantityValueTypeTest_types);

TYPED_TEST(QuantityValueTypeTest, PreIncrement) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(4), ++a);
    EXPECT_EQ(TypeParam(4), a);
}

TYPED_TEST(QuantityValueTypeTest, PostIncrement) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(3), a++);
    EXPECT_EQ(TypeParam(4), a);
}

TYPED_TEST(QuantityValueTypeTest, PreDecrement) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(2), --a);
    EXPECT_EQ(TypeParam(2), a);
}

TYPED_TEST(QuantityValueTypeTest, PostDecrement) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(3), a--);
    EXPECT_EQ(TypeParam(2), a);
}

TYPED_TEST(QuantityValueTypeTest, AddAssignment) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(5), a += TypeParam(2));
    EXPECT_EQ(TypeParam(5), a);
}

TYPED_TEST(QuantityValueTypeTest, SubAssignment) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(1), a -= TypeParam(2));
    EXPECT_EQ(TypeParam(1), a);
}

TYPED_TEST(QuantityValueTypeTest, MulAssignment) {
    TypeParam a(3);
    EXPECT_EQ(TypeParam(6), a *= 2);
    EXPECT_EQ(TypeParam(6), a);
}

TYPED_TEST(QuantityValueTypeTest, DivScalarAssignment) {
    TypeParam a(6);
    EXPECT_EQ(TypeParam(3), a /= 2);
    EXPECT_EQ(TypeParam(3), a);
}

TYPED_TEST(QuantityValueTypeTest, DivScalarWithRemainderAssignment) {
    TypeParam a(7);
    EXPECT_EQ(TypeParam(2), a /= 3);
    EXPECT_EQ(TypeParam(2), a);
}

TYPED_TEST(QuantityValueTypeTest, ModScalarAssignment) {
    TypeParam a(7);
    EXPECT_EQ(TypeParam(1), a %= 3);
    EXPECT_EQ(TypeParam(1), a);
}

TYPED_TEST(QuantityValueTypeTest, Add) {
    EXPECT_EQ(TypeParam(5), TypeParam(3) + TypeParam(2));
}

TYPED_TEST(QuantityValueTypeTest, Sub) {
    EXPECT_EQ(TypeParam(1), TypeParam(3) - TypeParam(2));
}

TYPED_TEST(QuantityValueTypeTest, Mul1) {
    EXPECT_EQ(TypeParam(6), TypeParam(3) * 2);
}

TYPED_TEST(QuantityValueTypeTest, Mul2) {
    EXPECT_EQ(TypeParam(6), 2 * TypeParam(3));
}

TYPED_TEST(QuantityValueTypeTest, DivScalar) {
    EXPECT_EQ(TypeParam(3), TypeParam(6) / 2);
}

TYPED_TEST(QuantityValueTypeTest, DivValue) {
    EXPECT_EQ(3, TypeParam(6) / TypeParam(2));
}

TYPED_TEST(QuantityValueTypeTest, DivScalarWithRemainder) {
    EXPECT_EQ(TypeParam(2), TypeParam(7) / 3);
}

TYPED_TEST(QuantityValueTypeTest, DivValueWithRemainder) {
    EXPECT_EQ(2, TypeParam(7) / TypeParam(3));
}

TYPED_TEST(QuantityValueTypeTest, ModScalar) {
    EXPECT_EQ(TypeParam(1), TypeParam(7) % 3);
}

TYPED_TEST(QuantityValueTypeTest, ModValue) {
    EXPECT_EQ(1, TypeParam(7) % TypeParam(3));
}

}
