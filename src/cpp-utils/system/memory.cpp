#include "memory.h"
#include <sys/mman.h>
#include <errno.h>
#include <stdexcept>
#include <cpp-utils/logging/logging.h>

using namespace cpputils::logging;

namespace cpputils {

DontSwapMemoryRAII::DontSwapMemoryRAII(const void *addr, size_t len)
: addr_(addr), len_(len) {
    const int result = ::mlock(addr_, len_);
    if (0 != result) {
        throw std::runtime_error("Error calling mlock. Errno: " + std::to_string(errno));
    }
}

DontSwapMemoryRAII::~DontSwapMemoryRAII() {
    const int result = ::munlock(addr_, len_);
    if (0 != result) {
        LOG(WARN, "Error calling munlock. Errno: {}", errno);
    }
}

}
