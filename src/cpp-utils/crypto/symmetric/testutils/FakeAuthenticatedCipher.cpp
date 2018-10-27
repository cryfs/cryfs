#include "FakeAuthenticatedCipher.h"

namespace cpputils {
    constexpr unsigned int FakeAuthenticatedCipher::KEYSIZE;
    constexpr unsigned int FakeAuthenticatedCipher::STRING_KEYSIZE;
    std::random_device FakeAuthenticatedCipher::random_;
}
