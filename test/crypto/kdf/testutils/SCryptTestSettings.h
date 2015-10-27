#ifndef MESSMER_CPPUTILS_TEST_CRYPTO_KDF_TESTUTILS_SCRYPTTESTSETTINGS_H
#define MESSMER_CPPUTILS_TEST_CRYPTO_KDF_TESTUTILS_SCRYPTTESTSETTINGS_H

#include <cstddef>
#include <cstdint>

struct SCryptTestSettings {
    constexpr static size_t SALT_LEN = 32; // Size of the salt
    constexpr static uint64_t N = 1024; // CPU/Memory cost
    constexpr static uint32_t r = 1; // Blocksize
    constexpr static uint32_t p = 1; // Parallelization
};

#endif
