#include "RandomPool.h"

namespace cpputils {

    constexpr size_t RandomPool::MIN_BUFFER_SIZE;
    constexpr size_t RandomPool::MAX_BUFFER_SIZE;
    std::mutex RandomPool::_mutex;
}