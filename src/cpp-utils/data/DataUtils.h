#pragma once
#ifndef MESSMER_CPPUTILS_DATA_DATAUTILS_H
#define MESSMER_CPPUTILS_DATA_DATAUTILS_H

#include "Data.h"

namespace cpputils {
    namespace DataUtils {
        //TODO Test

        //Return a new data object with the given size and initialize as much as possible with the given input data.
        //If the new data object is larger, then the remaining bytes will be zero filled.
        Data resize(const Data& data, size_t newSize);
    }
}

#endif
