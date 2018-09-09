#if defined(_MSC_VER)

#include "memory.h"
#include <Windows.h>
#include <stdexcept>
#include <cpp-utils/logging/logging.h>

using namespace cpputils::logging;

namespace cpputils {

DontSwapMemoryRAII::DontSwapMemoryRAII(void *addr, size_t len)
: addr_(addr), len_(len) {
	const BOOL result = ::VirtualLock(addr_, len_);
    if (!result) {
        throw std::runtime_error("Error calling VirtualLock. Errno: " + std::to_string(GetLastError()));
    }
}

DontSwapMemoryRAII::~DontSwapMemoryRAII() {
	const BOOL result = ::VirtualUnlock(addr_, len_);
    if (!result) {
        LOG(WARN, "Error calling VirtualUnlock. Errno: {}", GetLastError());
    }
}

}

#endif
