#pragma once
#ifndef MESSMER_CPPUTILS_VALUETYPE_CONFIGBUILDER_H_
#define MESSMER_CPPUTILS_VALUETYPE_CONFIGBUILDER_H_

namespace cpputils {

template<class Config> class ValueType;

template<class tag, class underlyingType, bool valueAccessEnabled, bool explicitValueConstructorEnabled, bool incrementAndDecrementEnabled>
struct ValueTypeConfig final {
private:
    /*
     * Accessors for building the ValueType<Config> class
     */
    friend class ValueType<ValueTypeConfig>;

    using underlying_type = underlyingType;
    static constexpr bool value_access_enabled() { return valueAccessEnabled; }
    static constexpr bool explicit_value_constructor_enabled() { return explicitValueConstructorEnabled; }
    static constexpr bool increment_and_decrement_enabled() { return incrementAndDecrementEnabled; }


public:
    /*
     * Setters for builder pattern
     */
    constexpr ValueTypeConfig<tag, underlyingType, true, explicitValueConstructorEnabled, incrementAndDecrementEnabled>
    enable_value_access() {
        static_assert(!valueAccessEnabled, "Can't call enable_value_access() twice");
        return {};
    }

    constexpr ValueTypeConfig<tag, underlyingType, valueAccessEnabled, true, incrementAndDecrementEnabled>
    enable_explicit_value_constructor() {
        static_assert(!explicitValueConstructorEnabled, "Can't call enable_explicit_value_constructor() twice");
        return {};
    }

    constexpr ValueTypeConfig<tag, underlyingType, valueAccessEnabled, explicitValueConstructorEnabled, true>
    enable_increment_and_decrement_operators() {
        static_assert(!incrementAndDecrementEnabled, "Can't call enable_increment_and_decrement_operators twice");
        return {};
    };

    using type = ValueType<ValueTypeConfig>;

private:
    constexpr ValueTypeConfig() {/* not meant for instantiation*/}
};

/**
 * Start building a value type.
 * tag: Some unique type to make sure the resulting value type is unique
 * underlyingType: The type of the underlying values
 */
template<class tag, class underlyingType>
inline constexpr ValueTypeConfig<tag, underlyingType, false, false, false> valueType() { return {}; }

}

#endif
