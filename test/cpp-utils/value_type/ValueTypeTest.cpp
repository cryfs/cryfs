#include <cpp-utils/value_type/ValueType.h>
#include <cpp-utils/value_type/ConfigBuilder.h>
#include <gtest/gtest.h>
#include <utility>

using cpputils::valueType;

struct Tag1{};
using AllEnabledValueType = decltype(valueType<Tag1, int>()
    .enable_explicit_value_constructor()
    .enable_value_access()
)::type;


/*
 * Test value() access
 */

template<class T, class Enable = void> struct has_value_access : std::false_type {};
template<class T> struct has_value_access<T, std::void_t<decltype(std::declval<T>().value())>> : std::true_type {
    using actual_result_type = std::result_of_t<decltype(&T::template value<void>)(T*)>;
    static_assert(std::is_same<typename T::underlying_type, actual_result_type>::value, "value() method returns wrong type");
};

struct Tag2{};
using ValueTypeWithValueAccess = decltype(valueType<Tag2, double>()
    .enable_explicit_value_constructor()
    .enable_value_access()
)::type;
struct Tag3{};
using ValueTypeWithoutValueAccess = decltype(valueType<Tag3, double>()
    .enable_explicit_value_constructor()
)::type;

static_assert(has_value_access<AllEnabledValueType>::value, "");
static_assert(has_value_access<ValueTypeWithValueAccess>::value, "");
static_assert(!has_value_access<ValueTypeWithoutValueAccess>::value, "");

TEST(ValueTypeTest, valueAccess){
    EXPECT_EQ(3, AllEnabledValueType(3).value());
    EXPECT_EQ(5.5, ValueTypeWithValueAccess(5.5).value());
}

struct Tag4{};
using ValueTypeWithIncrementAndDecrement = decltype(valueType<Tag4, int>()
    .enable_explicit_value_constructor()
    .enable_value_access()
    .enable_increment_and_decrement_operators()
)::type;
// TODO Test incrementAndDecrement