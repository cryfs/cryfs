#pragma once
#ifndef MESSMER_CPPUTILS_VALUETYPE_VALUETYPE_H_
#define MESSMER_CPPUTILS_VALUETYPE_VALUETYPE_H_

#include <functional>
#include <cpp-utils/assert/assert.h>

namespace cpputils {
namespace value_type {

/**
* This template simplifies generation of simple classes that wrap an id
* in a typesafe way. Namely, you can use it to create a very lightweight
* type that only offers equality comparators and hashing. Example:
*
*   struct MyIdType final : IdValueType<MyIdType, uint32_t> {
*     constexpr explicit MyIdType(uint32_t id): IdValueType(id) {}
*   };
*
* Then in the global top level namespace:
*
*   DEFINE_HASH_FOR_VALUE_TYPE(MyIdType);
*
* That's it - equality operators and hash functions are automatically defined
* for you, given the underlying type supports it.
*
* OrderedIdValueType: Use this instead of IdValueType if you need an ordering relation on your id type.
* This will define the operators
* - val < val
* - val > val
* - val <= val
* - val >= val
*
* QuantityValueType: Use this if you want a full-blown value type with arithmetics.
* Additionally to what OrderedIdValueType offers, this also defines:
* - ++val, val++
* - --val, val--
* - val += val (returns val)
* - val -= val (returns val)
* - val *= scalar (returns val)
* - val /= scalar (returns val)
* - val %= scalar (returns val)
* - val + val (returns val)
* - val - val (returns val)
* - val * scalar, scalar * val (returns val)
* - val / scalar (returns val)
* - val % scalar (returns val)
* - val / val (returns scalar)
* - val % val (returns scalar)
*/
template <class ConcreteType, class UnderlyingType>
class IdValueType {
public:
    using underlying_type = UnderlyingType;
    using concrete_type = ConcreteType;

    constexpr IdValueType(IdValueType&&) = default;
    constexpr IdValueType(const IdValueType&) = default;
    constexpr IdValueType& operator=(IdValueType&&) = default;
    constexpr IdValueType& operator=(const IdValueType&) = default;

protected:
    constexpr explicit IdValueType(underlying_type value) : value_(value) {
        static_assert(std::is_base_of<IdValueType<ConcreteType, UnderlyingType>, ConcreteType>::value,
                      "CRTP violated. First template parameter of this class must be the concrete class.");
    }
    constexpr underlying_type& underlying_value() const {
        return value_;
    }

    friend struct std::hash<ConcreteType>;

    friend constexpr bool operator==(ConcreteType lhs, ConcreteType rhs) {
        return lhs.value_ == rhs.value_;
    }

    friend constexpr bool operator!=(ConcreteType lhs, ConcreteType rhs) {
        return !operator==(lhs, rhs);
    }

    underlying_type value_;
};

#define DEFINE_HASH_FOR_VALUE_TYPE(ClassName)                   \
  namespace std {                                               \
  template <>                                                   \
  struct hash<ClassName> {                                      \
    size_t operator()(ClassName x) const {                      \
      return std::hash<ClassName::underlying_type>()(x.value_); \
    }                                                           \
  };                                                            \
  }


template <class ConcreteType, class UnderlyingType>
class OrderedIdValueType : public IdValueType<ConcreteType, UnderlyingType> {
protected:
    using IdValueType<ConcreteType, UnderlyingType>::IdValueType;

    friend constexpr bool operator<(ConcreteType lhs, ConcreteType rhs) {
        return lhs.value_ < rhs.value_;
    }

    friend constexpr bool operator>(ConcreteType lhs, ConcreteType rhs) {
        return lhs.value_ > rhs.value_;
    }

    friend constexpr bool operator>=(ConcreteType lhs, ConcreteType rhs) {
        return !operator<(lhs, rhs);
    }

    friend constexpr bool operator<=(ConcreteType lhs, ConcreteType rhs) {
        return !operator>(lhs, rhs);
    }
};


template <class ConcreteType, class UnderlyingType>
class QuantityValueType : public OrderedIdValueType<ConcreteType, UnderlyingType> {
protected:
    using OrderedIdValueType<ConcreteType, UnderlyingType>::OrderedIdValueType;

public:
    constexpr ConcreteType& operator++() {
        ++this->value_;
        return *this;
    }

    constexpr ConcreteType operator++(int) {
        ConcreteType tmp = *this;
        ++(*this);
        return tmp;
    }

    constexpr ConcreteType& operator--() {
        --this->value_;
        return *this;
    }

    constexpr ConcreteType operator--(int) {
        ConcreteType tmp = *this;
        --(*this);
        return tmp;
    }

    constexpr ConcreteType& operator+=(ConcreteType rhs) {
        this->value_ += rhs.value_;
        return *this;
    }

    constexpr ConcreteType& operator-=(ConcreteType rhs) {
        this->value_ -= rhs.value_;
        return *this;
    }

    constexpr ConcreteType& operator*=(UnderlyingType rhs) {
        this->value_ *= rhs;
        return *this;
    }

    constexpr ConcreteType& operator/=(UnderlyingType rhs) {
        this->value_ /= rhs;
        return *this;
    }

    constexpr ConcreteType& operator%=(UnderlyingType rhs) {
        this->value_ %= rhs;
        return *this;
    }

private:
    friend constexpr ConcreteType operator+(ConcreteType lhs, ConcreteType rhs) {
        return lhs += rhs;
    }

    friend constexpr ConcreteType operator-(ConcreteType lhs, ConcreteType rhs) {
        return lhs -= rhs;
    }

    friend constexpr ConcreteType operator*(ConcreteType lhs, UnderlyingType rhs) {
        return lhs *= rhs;
    }

    friend constexpr ConcreteType operator*(UnderlyingType lhs, ConcreteType rhs) {
        return rhs * lhs;
    }

    friend constexpr ConcreteType operator/(ConcreteType lhs, UnderlyingType rhs) {
        return lhs /= rhs;
    }

    friend constexpr UnderlyingType operator/(ConcreteType lhs, ConcreteType rhs) {
        return lhs.value_ / rhs.value_;
    }

    friend constexpr ConcreteType operator%(ConcreteType lhs, UnderlyingType rhs) {
        return lhs %= rhs;
    }

    friend constexpr UnderlyingType operator%(ConcreteType lhs, ConcreteType rhs) {
        return lhs.value_ % rhs.value_;
    }
};

}
}


#endif
