#include "time.h"

#if defined(_MSC_VER)
// Windows
// Implementation taken from https://stackoverflow.com/a/31335254

#include <Windows.h>
constexpr __int64 exp7 = 10000000i64;            //1E+7
constexpr __int64 exp9 = 1000000000i64;          //1E+9
constexpr __int64 w2ux = 116444736000000000i64;  //1.jan1601 to 1.jan1970
namespace cpputils {
namespace time {
struct timespec now() {
	__int64 wintime;
	GetSystemTimeAsFileTime((FILETIME*)&wintime);
	wintime -= w2ux;

	struct timespec spec;
	spec.tv_sec = wintime / exp7;
	spec.tv_nsec = wintime % exp7 * 100;
	return spec;
}
}
}

#elif defined(__MACH__) && !defined(CLOCK_REALTIME)
// OSX before 10.12 has no clock_gettime
// Implementation taken from: http://stackoverflow.com/a/9781275/829568
// Caution: The returned value is less precise than the returned value from a linux clock_gettime would be.

#include <sys/time.h>
namespace cpputils {
namespace time {

struct timespec now() {
	struct timeval now {};
	int rv = gettimeofday(&now, nullptr);
	if (rv) {
		throw std::runtime_error("gettimeofday failed with " + std::to_string(rv));
	}
	struct timespec result;
	result->tv_sec = now.tv_sec;
	result->tv_nsec = now.tv_usec * 1000;
	return now;
}

}
}

#else
// Linux or OSX with clock_gettime implementation

namespace cpputils {
namespace time {

struct timespec now() {
    struct timespec now{};
    clock_gettime(CLOCK_REALTIME, &now);
    return now;
}

}
}

#endif