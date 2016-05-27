#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_TIME_H
#define MESSMER_CPPUTILS_SYSTEM_TIME_H

#include <sys/types.h>
#include "clock_gettime.h"

namespace cpputils {
    namespace time {
        // TODO Test
        inline timespec now() {
            struct timespec now;
            clock_gettime(CLOCK_REALTIME, &now);
            return now;
        }
    }
}

#endif
