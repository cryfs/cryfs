#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_OSRANDOMGENERATOR_H
#define MESSMER_CPPUTILS_RANDOM_OSRANDOMGENERATOR_H

#include "RandomGenerator.h"
#include <vendor_cryptopp/osrng.h>

namespace cpputils {
    class OSRandomGenerator final : public RandomGenerator {
    public:
        OSRandomGenerator();

    protected:
        void _get(void *target, size_t bytes) override;

    private:
        DISALLOW_COPY_AND_ASSIGN(OSRandomGenerator);
    };

    inline OSRandomGenerator::OSRandomGenerator() {}

    inline void OSRandomGenerator::_get(void *target, size_t bytes) {
        CryptoPP::OS_GenerateRandomBlock(true, static_cast<CryptoPP::byte*>(target), bytes);
    }
}

#endif
