#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_MEMORY_H
#define MESSMER_CPPUTILS_SYSTEM_MEMORY_H

#include <cstdlib>
#include "../data/Data.h"

namespace cpputils {

// This allocator allocates memory that won't be swapped out to the disk, but will be kept in RAM
class UnswappableAllocator final : public Allocator {
public:
    void* allocate(size_t size) override;
    void free(void* data, size_t size) override;
};

}

#endif
