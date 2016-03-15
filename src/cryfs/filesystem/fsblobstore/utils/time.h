#pragma once
#ifndef MESSMER_CRYFS_FILESYTEM_FSBLOBSTORE_UTILS_TIME_H
#define MESSMER_CRYFS_FILESYTEM_FSBLOBSTORE_UTILS_TIME_H

#include <sys/types.h>
#include <cpp-utils/system/clock_gettime.h>

namespace cryfs {
    namespace fsblobstore {
        namespace time {

            inline timespec now() {
                struct timespec now;
                clock_gettime(CLOCK_REALTIME, &now);
                return now;
            }

        }
    }
}

#endif
