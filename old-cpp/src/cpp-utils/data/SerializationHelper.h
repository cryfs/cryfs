#pragma once
#ifndef MESSMER_CPPUTILS_DATA_SERIALIZATIONHELPER_H
#define MESSMER_CPPUTILS_DATA_SERIALIZATIONHELPER_H

#include <type_traits>
#include <cstring>
#include <cstdint>

namespace cpputils {

namespace details {

constexpr bool greater_than(size_t lhs, size_t rhs) {
    return lhs > rhs;
}

template<class DataType, class Enable = void> struct serialize;

// Specialize for 1-byte types for faster performance with direct pointer access (no memcpy).
template<class DataType>
struct serialize<DataType, typename std::enable_if<sizeof(DataType) == 1>::type> final {
    static_assert(std::is_pod<DataType>::value, "Can only serialize PODs");
    static void call(void *dst, const DataType &obj) {
        *static_cast<DataType *>(dst) = obj;
    }
};

// Specialize for larger types with memcpy because unaligned data accesses through pointers are undefined behavior.
template<class DataType>
struct serialize<DataType, typename std::enable_if<greater_than(sizeof(DataType), 1)>::type> final {
    static_assert(std::is_pod<DataType>::value, "Can only serialize PODs");
    static void call(void *dst, const DataType &obj) {
        std::memcpy(dst, &obj, sizeof(DataType));
    }
};

template<class DataType, class Enable = void> struct deserialize;

// Specialize for 1-byte types for faster performance with direct pointer access (no memcpy).
template<class DataType>
struct deserialize<DataType, typename std::enable_if<sizeof(DataType) == 1>::type> final {
    static_assert(std::is_pod<DataType>::value, "Can only serialize PODs");
    static DataType call(const void *src) {
        return *static_cast<const DataType *>(src);
    }
};

// Specialize for larger types with memcpy because unaligned data accesses through pointers are undefined behavior.
template<class DataType>
struct deserialize<DataType, typename std::enable_if<greater_than(sizeof(DataType), 1)>::type> final {
    static_assert(std::is_pod<DataType>::value, "Can only deserialize PODs");
    static DataType call(const void *src) {
        typename std::remove_const<DataType>::type result{};
        std::memcpy(&result, src, sizeof(DataType));
        return result;
    }
};

}

template<class DataType>
inline void serialize(void *dst, const DataType& obj) {
    return details::serialize<DataType>::call(dst, obj);
}

template<class DataType>
inline DataType deserialize(const void *src) {
    return details::deserialize<DataType>::call(src);
}

template<class DataType>
inline DataType deserializeWithOffset(const void* src, size_t offset) {
    return deserialize<DataType>(static_cast<const uint8_t*>(src) + offset);
}

}

#endif
