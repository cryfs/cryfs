#include "Math.h"

#include <cmath>

namespace blobstore {
namespace onblocks {
namespace utils {

uint32_t intPow(uint32_t base, uint32_t exponent) {
  uint32_t result = 1;
  for(uint32_t i = 0; i < exponent; ++i) {
    result *= base;
  }
  return result;
}

uint32_t ceilDivision(uint32_t dividend, uint32_t divisor) {
  return (dividend + divisor - 1)/divisor;
}

uint32_t maxZeroSubtraction(uint32_t minuend, uint32_t subtrahend) {
  if (minuend < subtrahend) {
    return 0u;
  }
  return minuend-subtrahend;
}

uint32_t ceilLog(uint32_t base, uint32_t value) {
  return std::ceil((long double)std::log(value)/(long double)std::log(base));
}

}
}
}
