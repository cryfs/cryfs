#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_CLOCKGETTIME_H
#define MESSMER_CPPUTILS_SYSTEM_CLOCKGETTIME_H

// Implements clock_gettime for Mac OS X (where it is not implemented by in the standard library)
// Source: http://stackoverflow.com/a/9781275/829568
// Caution: The returned value is less precise than the returned value from a linux clock_gettime would be.

#ifdef __MACH__
#include <sys/time.h>
#define CLOCK_REALTIME 0
inline int clock_gettime(int /*clk_id*/, struct timespec *result) {
    struct timeval now;
    int rv = gettimeofday(&now, nullptr);
    if (rv) {
        return rv;
    }
    result->tv_sec = now.tv_sec;
    result->tv_nsec = now.tv_sec * 1000;
    return 0;
}
#endif

#endif
