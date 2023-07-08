#include "diskspace.h"

namespace bf = boost::filesystem;

#if !defined(_MSC_VER)

#include <sys/statvfs.h>
#include <cerrno>

namespace cpputils {

uint64_t free_disk_space_in_bytes(const bf::path& location) {
	struct statvfs stat {};
	const int result = ::statvfs(location.string().c_str(), &stat);
	if (0 != result) {
		throw std::runtime_error("Error calling statvfs(). Errno: " + std::to_string(errno));
	}
	return stat.f_frsize*stat.f_bavail;
}

}

#else

#include <Windows.h>

namespace cpputils {

uint64_t free_disk_space_in_bytes(const bf::path& location) {
	ULARGE_INTEGER freeBytes;
	if (!GetDiskFreeSpaceEx(location.string().c_str(), &freeBytes, nullptr, nullptr)) {
		throw std::runtime_error("Error calling GetDiskFreeSpaceEx(). Error code: " + std::to_string(GetLastError()));
	}
	return freeBytes.QuadPart;
}

}

#endif
