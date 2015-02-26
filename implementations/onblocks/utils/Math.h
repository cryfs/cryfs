#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_UTILS_MATH_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_UTILS_MATH_H_

#include <cstdint>

namespace blobstore {
namespace onblocks {
namespace utils {

uint32_t intPow(uint32_t base, uint32_t exponent);
uint32_t ceilDivision(uint32_t dividend, uint32_t divisor);
uint32_t maxZeroSubtraction(uint32_t minuend, uint32_t subtrahend);

}
}
}


#endif
