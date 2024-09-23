#include <bits/types/struct_timeval.h>
#include <ctime>
#if !defined(_MSC_VER)

#include "filetime.h"
#include <array>
#include <errno.h>
#include <sys/stat.h>
#include <sys/time.h>

namespace cpputils {

int set_filetime(const char *filepath, timespec lastAccessTime, timespec lastModificationTime) {
	std::array<struct timeval, 2> casted_times{};
	TIMESPEC_TO_TIMEVAL(&casted_times[0], &lastAccessTime);
	TIMESPEC_TO_TIMEVAL(&casted_times[1], &lastModificationTime);
	const int retval = ::utimes(filepath, casted_times.data());
	if (0 == retval) {
		return 0;
	} else {
		return errno;
	}
}

int get_filetime(const char *filepath, timespec* lastAccessTime, timespec* lastModificationTime) {
	struct ::stat attrib{};
	const int retval = ::stat(filepath, &attrib);
	if (retval != 0) {
		return errno;
	}
	*lastAccessTime = attrib.st_atim;
	*lastModificationTime = attrib.st_mtim;
	return 0;
}

}

#endif
