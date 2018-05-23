#include <cpp-utils/value_type/ValueType.h>
#include <cpp-utils/value_type/ConfigBuilder.h>
#include <gtest/gtest.h>
#include <utility>
#include <unordered_set>
#include <set>

using cpputils::value_type::IdValueType;
using cpputils::value_type::OrderedIdValueType;
using cpputils::value_type::QuantityValueType;

namespace {

struct MyIdValueType : IdValueType<MyIdValueType, int64_t> {
    constexpr MyIdValueType(int64_t val): IdValueType(val) {}
};

struct MyOrderedIdValueType : OrderedIdValueType<MyOrderedIdValueType, int64_t> {
    constexpr MyOrderedIdValueType(int64_t val): OrderedIdValueType(val) {}
};

struct MyQuantityValueType : QuantityValueType<MyQuantityValueType, int64_t> {
    constexpr MyQuantityValueType(int64_t val): QuantityValueType(val) {}
};

}
DEFINE_HASH_FOR_VALUE_TYPE(MyIdValueType);
DEFINE_HASH_FOR_VALUE_TYPE(MyOrderedIdValueType);
DEFINE_HASH_FOR_VALUE_TYPE(MyQuantityValueType);
namespace {

template<class Type>
struct IdValueTypeTest_constexpr_test {
    static constexpr Type test_constructor = Type(5);
    static_assert(Type(5) == test_constructor, "");

    static constexpr Type test_copy_constructor = test_constructor;
    static_assert(Type(5) == test_copy_constructor, "");

    static constexpr Type test_copy_assignment = (Type(4) = 3);
    static_assert(test_copy_assignment == Type(3), "");

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

TYPED_TEST(IdValueTypeTest, MoveAssignment) {
    TypeParam obj(3);
    TypeParam obj2(2);
    obj2 = std::move(obj);
    EXPECT_TRUE(obj2 == TypeParam(3));
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







template<class Type>
struct OrderedIdValueTypeTest_constexpr_test {
   // TODO

    static constexpr bool success = true;
};
static_assert(OrderedIdValueTypeTest_constexpr_test<MyOrderedIdValueType>::success, "");
static_assert(OrderedIdValueTypeTest_constexpr_test<MyQuantityValueType>::success, "");

template<class Type> class OrderedIdValueTypeTest : public testing::Test {};
using OrderedIdValueTypeTest_types = testing::Types<MyOrderedIdValueType, MyQuantityValueType>;
TYPED_TEST_CASE(OrderedIdValueTypeTest, OrderedIdValueTypeTest_types);

// TODO Test cases for OrderedIdValueTypeTest

TYPED_TEST(OrderedIdValueTypeTest, Set) {
    std::set<TypeParam> set;
    set.insert(TypeParam(3));
    EXPECT_EQ(1u, set.count(TypeParam(3)));
}










template<class Type>
struct QuantityValueTypeTest_constexpr_test {
    // TODO

    static constexpr bool success = true;
};
static_assert(QuantityValueTypeTest_constexpr_test<MyQuantityValueType>::success, "");

template<class Type> class QuantityValueTypeTest : public testing::Test {};
using QuantityValueTypeTest_types = testing::Types<MyQuantityValueType>;
TYPED_TEST_CASE(QuantityValueTypeTest, QuantityValueTypeTest_types);

// TODO Test cases for QuantityValueTypeTest

}
