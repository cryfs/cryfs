#include "Random.h"
#include <mutex>

namespace cpputils {
    std::mutex Random::_mutex;
}
