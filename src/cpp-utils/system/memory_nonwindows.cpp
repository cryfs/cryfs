#if !defined(_MSC_VER)

#include "memory.h"
#include <sys/mman.h>
#include <errno.h>
#include <stdexcept>
#include <cpp-utils/logging/logging.h>

using namespace cpputils::logging;

namespace cpputils {

void* UnswappableAllocator::allocate(size_t size) {
    void* data = DefaultAllocator().allocate(size);
    const int result = ::mlock(data, size);
    if (0 != result) {
        throw std::runtime_error("Error calling mlock. Errno: " + std::to_string(errno));
    }
    return data;
}

void UnswappableAllocator::free(void* data, size_t size) {
    const int result = ::munlock(data, size);
    if (0 != result) {
        LOG(WARN, "Error calling munlock. Errno: {}", errno);
    }

    // overwrite the memory with zeroes before we free it
    std::memset(data, 0, size);

    DefaultAllocator().free(data, size);
}

}

#endif
