#include "FakeAuthenticatedCipher.h"

namespace cpputils {
    constexpr unsigned int FakeKey::BINARY_LENGTH;

    std::random_device FakeAuthenticatedCipher::random_;
}
