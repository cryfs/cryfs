#include "Hash.h"
#include <cpp-utils/random/Random.h>
#include <vendor_cryptopp/sha.h>

using CryptoPP::SHA512;

namespace cpputils {
namespace hash {

Hash hash(const Data& data, Salt salt) {
  SHA512 hasher; // NOLINT (workaround for clang-warning in libcrypto++)
  hasher.Update(static_cast<const CryptoPP::byte*>(salt.data()), Salt::BINARY_LENGTH);
  hasher.Update(static_cast<const CryptoPP::byte*>(data.data()), data.size());

  Digest digest = Digest::Null();
  hasher.Final(static_cast<CryptoPP::byte*>(digest.data()));

  return Hash{
      digest,
      salt
  };
}

Salt generateSalt() {
  return Random::PseudoRandom().getFixedSize<8>();
}

}
}
