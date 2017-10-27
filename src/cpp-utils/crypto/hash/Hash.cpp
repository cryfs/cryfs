#include "Hash.h"
#include <cpp-utils/random/Random.h>
#include <cryptopp/sha.h>

using cpputils::Random;
using CryptoPP::SHA512;

namespace cpputils {
namespace hash {

Hash hash(const Data& data, Salt salt) {
  SHA512 hasher; // NOLINT (workaround for clang-warning in libcrypto++)
  hasher.Update((CryptoPP::byte*)salt.data(), Salt::BINARY_LENGTH);
  hasher.Update((CryptoPP::byte*)data.data(), data.size());

  Digest digest = Digest::Null();
  hasher.Final((CryptoPP::byte*)digest.data());

  return Hash{
      .digest = std::move(digest),
      .salt = std::move(salt)
  };
}

Salt generateSalt() {
  return Random::PseudoRandom().getFixedSize<8>();
}

}
}
