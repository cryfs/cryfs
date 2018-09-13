#if defined(_MSC_VER)

#include "memory.h"
#include <Windows.h>
#include <stdexcept>
#include <cpp-utils/logging/logging.h>

using namespace cpputils::logging;

namespace cpputils {

void* UnswappableAllocator::allocate(size_t size) {
	// VirtualAlloc allocates memory in full pages. This is needed, because VirtualUnlock unlocks full pages
	// and might otherwise unlock unrelated memory of other allocations.
    
	// allocate pages
	void* data = ::VirtualAlloc(nullptr, size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
	if (nullptr == data) {
		throw std::runtime_error("Error calling VirtualAlloc. Errno: " + std::to_string(GetLastError()));
	}

	// lock allocated pages into RAM
	const BOOL success = ::VirtualLock(data, size);
	if (!success) {
		throw std::runtime_error("Error calling VirtualLock. Errno: " + std::to_string(GetLastError()));
	}

    return data;
}

void UnswappableAllocator::free(void* data, size_t size) {
	// overwrite the memory with zeroes before we free it
	std::memset(data, 0, size);

	// unlock allocated pages from RAM
	BOOL success = ::VirtualUnlock(data, size);
	if (!success) {
		throw std::runtime_error("Error calling VirtualUnlock. Errno: " + std::to_string(GetLastError()));
	}

	// free allocated pages
	success = ::VirtualFree(data, 0, MEM_RELEASE);
	if (!success) {
		throw std::runtime_error("Error calling VirtualFree. Errno: " + std::to_string(GetLastError()));
	}
}

}

#endif
