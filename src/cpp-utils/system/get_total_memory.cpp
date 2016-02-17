#include "get_total_memory.h"
#include <sys/sysctl.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdexcept>

namespace cpputils{
    namespace system {
        uint64_t get_total_memory() {
            uint64_t mem;
#ifdef __APPLE__
            size_t size = sizeof(mem);
  int result = sysctlbyname("hw.memsize", &mem, &size, nullptr, 0);
  if (0 != result) {
    throw std::runtime_error("sysctlbyname syscall failed");
  }
#elif __linux__
            long numRAMPages = sysconf(_SC_PHYS_PAGES);
            long pageSize = sysconf(_SC_PAGESIZE);
            mem = numRAMPages * pageSize;
#else
#error Not supported on windows yet, TODO http://stackoverflow.com/a/2513561/829568
#endif
            return mem;
        }
    }
}
