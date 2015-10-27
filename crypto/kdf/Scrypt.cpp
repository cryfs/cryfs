#include "Scrypt.h"

namespace cpputils {
    constexpr size_t SCryptDefaultSettings::SALT_LEN;
    constexpr uint64_t SCryptDefaultSettings::N;
    constexpr uint32_t SCryptDefaultSettings::r;
    constexpr uint32_t SCryptDefaultSettings::p;

    constexpr size_t SCryptParanoidSettings::SALT_LEN;
    constexpr uint64_t SCryptParanoidSettings::N;
    constexpr uint32_t SCryptParanoidSettings::r;
    constexpr uint32_t SCryptParanoidSettings::p;
}