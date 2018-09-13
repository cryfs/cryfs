#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_MEMORY_H
#define MESSMER_CPPUTILS_SYSTEM_MEMORY_H

#include <cstdlib>
#include "../data/Data.h"

namespace cpputils {

/**
* Allocator for security relevant memory like key data.
* The operating system will be given a hint that this memory shouldn't be swapped out to disk
* (which is, however, only a hint and might be ignored),
* and we'll make sure the memory is zeroed-out when deallocated.
*/
class UnswappableAllocator final : public Allocator {
public:
    void* allocate(size_t size) override;
    void free(void* data, size_t size) override;
};

}

#endif
