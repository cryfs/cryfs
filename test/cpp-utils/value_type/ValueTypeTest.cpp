#include <cpp-utils/value_type/ValueType.h>
#include <gtest/gtest.h>

using cpputils::ValueType;

class AllEnabledConfig final {
    using underlying_type = int;
    static constexpr bool allow_value_access() { return true; }
};

using AllEnabledValueType = ValueType<AllEnabledConfig>;

template<class T, class Enable = void> struct has_value_access : std::false_type {};
template<class T> struct has_value_access<T, std::void_t<T::value()>> : std::true_type {
    static_assert(std::is_same<typename T::underlying_type, std::result_of_t<T::value()>>::value, "value() method returns wrong type");
};

struct ConfigWithValueAccess final {
    using underlying_type = int;
    static constexpr bool allow_value_access() { return true; }
};

struct ConfigWithoutValueAccess final {
    using underlying_type = int;
    static constexpr bool allow_value_access() { return false; }
};

static_assert(has_value_access<ValueType<AllEnabledConfig>>::value, "");
static_assert(has_value_access<ValueType<ConfigWithValueAccess>>::value, "");
static_assert(!has_value_access<ValueType<ConfigWithoutValueAccess>>::value, "");