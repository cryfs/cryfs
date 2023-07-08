#pragma once
#ifndef MESSMER_FSPP_IMPL_PROFILER_H
#define MESSMER_FSPP_IMPL_PROFILER_H

#include <atomic>
#include <chrono>
#include <cpp-utils/macros.h>

namespace fspp {
    class Profiler final {
    public:
        Profiler(std::atomic_uint_fast64_t *targetForAddingNanosec);
        ~Profiler();

    private:
        std::atomic_uint_fast64_t *_targetForAddingNanosec;
        std::chrono::high_resolution_clock::time_point _beginTime;

        DISALLOW_COPY_AND_ASSIGN(Profiler);
    };

    inline Profiler::Profiler(std::atomic_uint_fast64_t *targetForAddingNanosec)
            : _targetForAddingNanosec(targetForAddingNanosec), _beginTime(std::chrono::high_resolution_clock::now()) {
    }

    inline Profiler::~Profiler() {
        const uint64_t timeDiff = std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::high_resolution_clock::now() - _beginTime).count();
        *_targetForAddingNanosec += timeDiff;
    }
}

#endif
