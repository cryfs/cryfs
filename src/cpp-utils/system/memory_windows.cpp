#if defined(_MSC_VER)

#include "memory.h"
#include <Windows.h>
#include <stdexcept>
#include <cpp-utils/logging/logging.h>

using namespace cpputils::logging;

namespace cpputils {

void* UnswappableAllocator::allocate(size_t size) {
    void* data = DefaultAllocator().allocate(size);
	const BOOL result = ::VirtualLock(data, size);
    if (!result) {
        throw std::runtime_error("Error calling VirtualLock. Errno: " + std::to_string(GetLastError()));
    }
    return data;
}

void UnswappableAllocator::free(void* data, size_t size) {
	const BOOL result = ::VirtualUnlock(data, size);
    if (!result) {
        LOG(WARN, "Error calling VirtualUnlock. Errno: {}", GetLastError());
    }

    // overwrite the memory with zeroes before we free it
    std::memset(data, 0, size);

    DefaultAllocator().free(data, size);
}

}

#endif
