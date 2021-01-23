#pragma once
#ifndef MESSMER_CPPUTILS_DATA_DATAFIXTURE_H_
#define MESSMER_CPPUTILS_DATA_DATAFIXTURE_H_

#include "Data.h"
#include "FixedSizeData.h"

namespace cpputils {

class DataFixture final {
public:
  static Data generate(size_t size, unsigned long long int seed = 1);

  //TODO Test
  template<size_t SIZE> static FixedSizeData<SIZE> generateFixedSize(long long int seed = 1);
};

template<size_t SIZE> FixedSizeData<SIZE> DataFixture::generateFixedSize(long long int seed) {
  Data data = generate(SIZE, seed);
  auto result = FixedSizeData<SIZE>::Null();
  std::memcpy(result.data(), data.data(), SIZE);
  return result;
}

}

#endif
