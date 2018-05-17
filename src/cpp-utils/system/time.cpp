#include "time.h"

#if defined(__MACH__) && !defined(CLOCK_REALTIME)

// Implements clock_gettime for Mac OS X before 10.12 (where it is not implemented by in the standard library)
// Source: http://stackoverflow.com/a/9781275/829568
// Caution: The returned value is less precise than the returned value from a linux clock_gettime would be.
#include <sys/time.h>
#define CLOCK_REALTIME 0
namespace {
int clock_gettime(int /*clk_id*/, struct timespec *result) {
    struct timeval now;
    int rv = gettimeofday(&now, nullptr);
    if (rv) {
        return rv;
    }
    result->tv_sec = now.tv_sec;
    result->tv_nsec = now.tv_usec * 1000;
    return 0;
}
}

#endif

namespace cpputils {
namespace time {

struct timespec now() {
    struct timespec now{};
    clock_gettime(CLOCK_REALTIME, &now);
    return now;
}

}
}
