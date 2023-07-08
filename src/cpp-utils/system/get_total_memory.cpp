#include "get_total_memory.h"
#include <stdexcept>
#include <string>

#if defined(__APPLE__)

#include <sys/types.h>
#include <sys/sysctl.h>

namespace cpputils {
	namespace system {
		uint64_t get_total_memory() {
			uint64_t mem;
			size_t size = sizeof(mem);
			int result = sysctlbyname("hw.memsize", &mem, &size, nullptr, 0);
			if (0 != result) {
				throw std::runtime_error("sysctlbyname syscall failed");
			}
			return mem;
		}
	}
}

#elif defined(__linux__) || defined(__FreeBSD__)

#include <unistd.h>

namespace cpputils {
	namespace system {
		uint64_t get_total_memory() {
			const long numRAMPages = sysconf(_SC_PHYS_PAGES);
			const long pageSize = sysconf(_SC_PAGESIZE);
			return numRAMPages * pageSize;
		}
	}
}

#elif defined(_MSC_VER)

#include <Windows.h>

namespace cpputils {
	namespace system {
		uint64_t get_total_memory() {
			MEMORYSTATUSEX status;
			status.dwLength = sizeof(status);
			if (!::GlobalMemoryStatusEx(&status)) {
				throw std::runtime_error("Couldn't get system memory information. Error code: " + std::to_string(GetLastError()));
			}
			return status.ullTotalPhys;
		}
	}
}

#else
#error Unsupported platform
#endif

