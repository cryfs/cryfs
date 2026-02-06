#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOM_H
#define MESSMER_CPPUTILS_RANDOM_RANDOM_H

#include "CsprngPool.h"
#include "OSRandomGenerator.h"
#include "../data/FixedSizeData.h"
#include "../data/Data.h"
#include <mutex>

namespace cpputils {
    class Random final {
    public:
        static CsprngPool *Csprng() {
            const std::unique_lock <std::mutex> lock(_mutex);
            static CsprngPool random;
            return &random;
        }

        static OSRandomGenerator *OSRandom() {
            const std::unique_lock <std::mutex> lock(_mutex);
            static OSRandomGenerator random;
            return &random;
        }

    private:
        static std::mutex _mutex;

        DISALLOW_COPY_AND_ASSIGN(Random);
    };
}

#endif
