#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_UTILS_MATH_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_UTILS_MATH_H_

#include <cstdint>
#include <cmath>

namespace blobstore {
namespace onblocks {
namespace utils {

template<typename INT_TYPE>
inline INT_TYPE intPow(INT_TYPE base, INT_TYPE exponent) {
    INT_TYPE result = 1;
    for(INT_TYPE i = 0; i < exponent; ++i) {
        result *= base;
    }
    return result;
}

template<typename INT_TYPE>
inline INT_TYPE ceilDivision(INT_TYPE dividend, INT_TYPE divisor) {
    return (dividend + divisor - 1)/divisor;
}

template<typename INT_TYPE>
inline INT_TYPE maxZeroSubtraction(INT_TYPE minuend, INT_TYPE subtrahend) {
    if (minuend < subtrahend) {
        return 0u;
    }
    return minuend-subtrahend;
}

template<typename INT_TYPE>
inline INT_TYPE ceilLog(INT_TYPE base, INT_TYPE value) {
    return std::ceil(static_cast<long double>(std::log(value))/static_cast<long double>(std::log(base)));
}

}
}
}

#endif
