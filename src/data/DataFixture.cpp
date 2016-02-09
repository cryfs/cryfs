#include "DataFixture.h"

namespace cpputils {
  Data DataFixture::generate(size_t size, long long int seed) {
    Data result(size);
    long long int val = seed;
    for(size_t i=0; i<size/sizeof(long long int); ++i) {
      //MMIX linear congruential generator
      val *= 6364136223846793005L;
      val += 1442695040888963407;
      reinterpret_cast<long long int*>(result.data())[i] = val;
    }
    uint64_t alreadyWritten = (size/sizeof(long long int))*sizeof(long long int);
    val *= 6364136223846793005L;
    val += 1442695040888963407;
    char *remainingBytes = reinterpret_cast<char*>(&val);
    //Fill remaining bytes
    for(size_t i=0; i<size-alreadyWritten; ++i) {
      reinterpret_cast<char*>(result.data())[alreadyWritten + i] = remainingBytes[i];
    }
    return result;
  }
}
