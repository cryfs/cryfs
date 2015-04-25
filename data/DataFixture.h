#pragma once
#ifndef MESSMER_CPP_UTILS_DATA_DATAFIXTURE_H_
#define MESSMER_CPP_UTILS_DATA_DATAFIXTURE_H_

#include "Data.h"

namespace cpputils {

class DataFixture {
public:
  static Data generate(size_t size, long long int seed = 1);
};

}

#endif
