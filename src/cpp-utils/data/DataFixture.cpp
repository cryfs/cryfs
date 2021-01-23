#include "DataFixture.h"
#include "SerializationHelper.h"

namespace cpputils {
  Data DataFixture::generate(size_t size, unsigned long long int seed) {
    Data result(size);
    unsigned long long int val = seed;
    for(size_t i=0; i<size/sizeof(unsigned long long int); ++i) {
      //MMIX linear congruential generator
      val *= 6364136223846793005L;
      val += 1442695040888963407;
      serialize<unsigned long long int>(result.dataOffset(i*sizeof(unsigned long long int)), val);
    }
    uint64_t alreadyWritten = (size/sizeof(unsigned long long int))*sizeof(unsigned long long int);
    val *= 6364136223846793005L;
    val += 1442695040888963407;
    unsigned char *remainingBytes = reinterpret_cast<unsigned char*>(&val);
    //Fill remaining bytes
    for(size_t i=0; i<size-alreadyWritten; ++i) {
      serialize<unsigned char>(result.dataOffset(alreadyWritten + i), remainingBytes[i]);
    }
    return result;
  }
}
