#pragma once
#ifndef MESSMER_CPPUTILS_VALUETYPE_VALUETYPE_H_
#define MESSMER_CPPUTILS_VALUETYPE_VALUETYPE_H_

#include <functional>
#include <cpp-utils/assert/assert.h>

namespace cpputils {

namespace details {
// The Helper type is to allow the enable_if to depend on another type, say a local template type of the method.
// If enable_if depended only on class template parameters, it wouldn't work because they're already deduced when deducing the method.
template<class Helper, bool Condition, class Type>
using enable_if_t = std::enable_if_t<std::is_void<Helper>::value && Condition, Type>;
}

// TODO Test
template<class Config>
class ValueType final {
public:
    using underlying_type = typename Config::underlying_type;

    template<class U = void>
    constexpr explicit ValueType(details::enable_if_t<U, Config::explicit_value_constructor_enabled(), underlying_type> value)
    : _value(value) {}

    template<class U = void>
    constexpr details::enable_if_t<U, Config::value_access_enabled(), underlying_type> value() const {
        return _value;
    }

    template<class U = void>
    constexpr details::enable_if_t<U, Config::increment_and_decrement_enabled(), ValueType&> operator++() {
        ++_value;
        return *this;
    }

    template<class U = void>
    constexpr details::enable_if_t<U, Config::increment_and_decrement_enabled(), ValueType> operator++(int) {
        ValueType<Config> tmp = *this;
        ++(*this);
        return tmp;
    }

    template<class U = void>
    constexpr details::enable_if_t<U, Config::increment_and_decrement_enabled(), ValueType&> operator--() {
        --_value;
        return *this;
    }

    template<class U = void>
    constexpr details::enable_if_t<U, Config::increment_and_decrement_enabled(), ValueType> operator--(int) {
        ValueType<Config> tmp = *this;
        --(*this);
        return tmp;
    }

    constexpr ValueType& operator+=(ValueType rhs);
    constexpr ValueType& operator-=(ValueType rhs);
    constexpr ValueType& operator*=(underlying_type rhs);
    constexpr ValueType& operator/=(underlying_type rhs);
    constexpr ValueType& operator%=(underlying_type rhs);

private:
    friend struct std::hash<ValueType>;
    underlying_type _value;
};

/*template<class Config>
constexpr ValueType<Config> operator "" _bytes(unsigned long long int value);*/

template<class Config> constexpr ValueType<Config> operator+(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr ValueType<Config> operator-(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr ValueType<Config> operator*(ValueType<Config> lhs, typename Config::underlying_type rhs);
template<class Config> constexpr ValueType<Config> operator*(typename Config::underlying_type lhs, ValueType<Config> rhs);
template<class Config> constexpr ValueType<Config> operator/(ValueType<Config> lhs, typename Config::underlying_type rhs);
template<class Config> constexpr typename Config::underlying_type operator/(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr ValueType<Config> operator%(ValueType<Config> lhs, typename Config::underlying_type rhs);
template<class Config> constexpr typename Config::underlying_type operator%(ValueType<Config> lhs, ValueType<Config> rhs);

template<class Config> constexpr bool operator==(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr bool operator!=(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr bool operator<(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr bool operator>(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr bool operator<=(ValueType<Config> lhs, ValueType<Config> rhs);
template<class Config> constexpr bool operator>=(ValueType<Config> lhs, ValueType<Config> rhs);


/*
 * Implementation follows
 */

/*template<class Config>
inline constexpr ValueType<Config> operator "" _bytes(unsigned long long int value) {
    return ValueType<Config>(value);
}*/


template<class Config>
inline constexpr ValueType<Config>& ValueType<Config>::operator+=(ValueType<Config> rhs) {
    _value += rhs._value;
    return *this;
}

template<class Config>
inline constexpr ValueType<Config>& ValueType<Config>::operator-=(ValueType<Config> rhs) {
    _value -= rhs._value;
    return *this;
}

template<class Config>
inline constexpr ValueType<Config>& ValueType<Config>::operator*=(typename Config::underlying_type rhs) {
    _value *= rhs;
    return *this;
}

template<class Config>
inline constexpr ValueType<Config>& ValueType<Config>::operator/=(typename Config::underlying_type rhs) {
    _value /= rhs;
    return *this;
}

template<class Config>
inline constexpr ValueType<Config>& ValueType<Config>::operator%=(typename Config::underlying_type rhs) {
    _value %= rhs;
    return *this;
}

template<class Config>
inline constexpr ValueType<Config> operator+(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs += rhs;
}

template<class Config>
inline constexpr ValueType<Config> operator-(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs -= rhs;
}

template<class Config>
inline constexpr ValueType<Config> operator*(ValueType<Config> lhs, typename Config::underlying_type rhs) {
    return lhs *= rhs;
}

template<class Config>
inline constexpr ValueType<Config> operator*(typename Config::underlying_type lhs, ValueType<Config> rhs) {
    return rhs * lhs;
}

template<class Config>
inline constexpr ValueType<Config> operator/(ValueType<Config> lhs, typename Config::underlying_type rhs) {
    return lhs /= rhs;
}

template<class Config>
inline constexpr typename Config::underlying_type operator/(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs.value() / rhs.value();
}

template<class Config>
inline constexpr ValueType<Config> operator%(ValueType<Config> lhs, typename Config::underlying_type rhs) {
    return lhs %= rhs;
}

template<class Config>
inline constexpr typename Config::underlying_type operator%(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs.value() % rhs.value();
}


template<class Config>
inline constexpr bool operator==(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs.value() == rhs.value();
}

template<class Config>
inline constexpr bool operator!=(ValueType<Config> lhs, ValueType<Config> rhs) {
    return !operator==(lhs, rhs);
}

template<class Config>
inline constexpr bool operator<(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs.value() < rhs.value();
}

template<class Config>
inline constexpr bool operator>(ValueType<Config> lhs, ValueType<Config> rhs) {
    return lhs.value() > rhs.value();
}

template<class Config>
inline constexpr bool operator<=(ValueType<Config> lhs, ValueType<Config> rhs) {
    return !operator>(lhs, rhs);
}

template<class Config>
inline constexpr bool operator>=(ValueType<Config> lhs, ValueType<Config> rhs) {
    return !operator<(lhs, rhs);
}

}

namespace std {
    template<class Config>
    struct hash<cpputils::ValueType<Config>> {
        constexpr hash() = default;
        constexpr size_t operator()(cpputils::ValueType<Config> v) {
            return v._value;
        }
    };
}

#endif
