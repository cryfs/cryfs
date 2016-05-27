#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_TIME_H
#define MESSMER_CPPUTILS_SYSTEM_TIME_H

#include <sys/types.h>
#include "clock_gettime.h"

// TODO Test

namespace cpputils {
    namespace time {

        inline timespec now() {
            struct timespec now;
            clock_gettime(CLOCK_REALTIME, &now);
            return now;
        }

    }
}

inline bool operator==(const timespec &lhs, const timespec &rhs) {
    return lhs.tv_sec == rhs.tv_sec && lhs.tv_nsec == rhs.tv_nsec;
}

inline bool operator<(const timespec &lhs, const timespec &rhs) {
    return lhs.tv_sec < rhs.tv_sec || (lhs.tv_sec == rhs.tv_sec && lhs.tv_nsec < rhs.tv_nsec);
}

inline bool operator>(const timespec &lhs, const timespec &rhs) {
    return lhs.tv_sec > rhs.tv_sec || (lhs.tv_sec == rhs.tv_sec && lhs.tv_nsec > rhs.tv_nsec);
}

inline bool operator!=(const timespec &lhs, const timespec &rhs) {
    return !operator==(lhs, rhs);
}

inline bool operator<=(const timespec &lhs, const timespec &rhs) {
    return !operator>(lhs, rhs);
}

inline bool operator>=(const timespec &lhs, const timespec &rhs) {
    return !operator<(lhs, rhs);
}

#endif
