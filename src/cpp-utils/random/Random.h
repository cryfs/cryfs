#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOM_H
#define MESSMER_CPPUTILS_RANDOM_RANDOM_H

#include "PseudoRandomPool.h"
#include "OSRandomGenerator.h"
#include "../data/FixedSizeData.h"
#include "../data/Data.h"
#include <mutex>

namespace cpputils {
    class Random final {
    public:
        static PseudoRandomPool &PseudoRandom() {
            const std::unique_lock <std::mutex> lock(_mutex);
            static PseudoRandomPool random;
            return random;
        }

        static OSRandomGenerator &OSRandom() {
            const std::unique_lock <std::mutex> lock(_mutex);
            static OSRandomGenerator random;
            return random;
        }

    private:
        static std::mutex _mutex;

        DISALLOW_COPY_AND_ASSIGN(Random);
    };
}

#endif
