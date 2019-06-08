#if !defined(_MSC_VER)

#include "filetime.h"
#include <utime.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <errno.h>
#include <array>
#include <cpp-utils/system/stat.h>

namespace cpputils {

int set_filetime(const char *filepath, timespec lastAccessTime, timespec lastModificationTime) {
	std::array<struct timeval, 2> casted_times{};
	TIMESPEC_TO_TIMEVAL(&casted_times[0], &lastAccessTime);
	TIMESPEC_TO_TIMEVAL(&casted_times[1], &lastModificationTime);
	int retval = ::utimes(filepath, casted_times.data());
	if (0 == retval) {
		return 0;
	} else {
		return errno;
	}
}

int get_filetime(const char *filepath, timespec* lastAccessTime, timespec* lastModificationTime) {
	struct ::stat attrib{};
	int retval = ::stat(filepath, &attrib);
	if (retval != 0) {
		return errno;
	}
	*lastAccessTime = attrib.st_atim;
	*lastModificationTime = attrib.st_mtim;
	return 0;
}

}

#endif
