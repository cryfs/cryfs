#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_OSRANDOMGENERATOR_H
#define MESSMER_CPPUTILS_RANDOM_OSRANDOMGENERATOR_H

#include "RandomGenerator.h"
#include <cryptopp/cryptopp/osrng.h>

namespace cpputils {
    class OSRandomGenerator final : public RandomGenerator {
    public:
        OSRandomGenerator();

    protected:
        void get(void *target, size_t bytes) override;
    };

    inline OSRandomGenerator::OSRandomGenerator() {}

    inline void OSRandomGenerator::get(void *target, size_t bytes) {
        CryptoPP::OS_GenerateRandomBlock(true, (byte*)target, bytes);
    }
}

#endif
