#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_MEMORY_H
#define MESSMER_CPPUTILS_SYSTEM_MEMORY_H

#include <cstdlib>

namespace cpputils {

// While this RAII object exists, it locks a given memory address into RAM,
// i.e. tells the operating system not to swap it.
class DontSwapMemoryRAII final {
public:
    DontSwapMemoryRAII(void* addr, size_t len);
    ~DontSwapMemoryRAII();

private:
    void* const addr_;
    const size_t len_;
};

}

#endif
