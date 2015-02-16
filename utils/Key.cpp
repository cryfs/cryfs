#include <cryptopp/cryptopp/hex.h>
#include <cryptopp/cryptopp/osrng.h>
#include <messmer/blockstore/utils/Key.h>

#include <cstring>

using CryptoPP::ArraySource;
using CryptoPP::ArraySink;
using CryptoPP::StringSink;
using CryptoPP::StringSource;
using CryptoPP::HexEncoder;
using CryptoPP::HexDecoder;
using CryptoPP::AutoSeededRandomPool;

using std::string;

namespace blockstore {

constexpr unsigned int Key::KEYLENGTH_BINARY;
constexpr unsigned int Key::KEYLENGTH_STRING;

Key::Key() {
}

Key::~Key() {
}

AutoSeededRandomPool &RandomPool() {
  static AutoSeededRandomPool singleton;
  return singleton;
}

Key Key::CreateRandomKey() {
  Key result;
  RandomPool().GenerateBlock(result._key, KEYLENGTH_BINARY);
  return result;
}

Key Key::FromString(const std::string &key) {
  assert(key.size() == KEYLENGTH_STRING);
  Key result;
  StringSource(key, true,
    new HexDecoder(new ArraySink(result._key, KEYLENGTH_BINARY))
  );
  return result;
}

string Key::ToString() const {
  string result;
  ArraySource(_key, KEYLENGTH_BINARY, true,
    new HexEncoder(new StringSink(result))
  );
  assert(result.size() == KEYLENGTH_STRING);
  return result;
}

const unsigned char *Key::data() const {
  return _key;
}

bool operator==(const Key &lhs, const Key &rhs) {
  return 0 == std::memcmp(lhs.data(), rhs.data(), Key::KEYLENGTH_BINARY);
}

bool operator!=(const Key &lhs, const Key &rhs) {
  return !operator==(lhs, rhs);
}

void Key::ToBinary(void *target) const {
  std::memcpy(target, _key, KEYLENGTH_BINARY);
}

Key Key::FromBinary(const void *source) {
  Key result;
  std::memcpy(result._key, source, KEYLENGTH_BINARY);
  return result;
}

}
