#include "DataUtils.h"
#include "cpp-utils/data/Data.h"
#include <algorithm>
#include <cstddef>
#include <cstring>

namespace cpputils {
    namespace DataUtils {
        Data resize(const Data& data, size_t newSize) {
            Data newData(newSize);
            newData.FillWithZeroes(); // TODO Only fill region after copied old data with zeroes
            std::memcpy(newData.data(), data.data(), std::min(newData.size(), data.size()));
            return newData;
        }
    }
}
