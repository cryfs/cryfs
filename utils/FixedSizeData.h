#pragma once
#ifndef BLOCKSTORE_UTILS_data_H_
#define BLOCKSTORE_UTILS_data_H_

#include <cryptopp/cryptopp/hex.h>
#include <cryptopp/cryptopp/osrng.h>
#include <string>
#include <cstring>

namespace blockstore {

template<int SIZE>
class FixedSizeData {
public:
  //Non-virtual destructor because we want objects to be small
  ~FixedSizeData() {}

  static constexpr unsigned int BINARY_LENGTH = SIZE;
  static constexpr unsigned int STRING_LENGTH = 2 * BINARY_LENGTH; // Hex encoding

  static FixedSizeData<SIZE> CreateRandom();

  static FixedSizeData<SIZE> FromString(const std::string &data);
  std::string ToString() const;

  static FixedSizeData<SIZE> FromBinary(const void *source);
  void ToBinary(void *target) const;

  const unsigned char *data() const;

private:
  FixedSizeData() {}
  static CryptoPP::AutoSeededRandomPool &RandomPool();

  unsigned char _data[BINARY_LENGTH];
};

template<int SIZE> bool operator==(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs);
template<int SIZE> bool operator!=(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs);

// ----- Implementation -----

template<int SIZE> constexpr unsigned int FixedSizeData<SIZE>::BINARY_LENGTH;
template<int SIZE> constexpr unsigned int FixedSizeData<SIZE>::STRING_LENGTH;

template<int SIZE>
CryptoPP::AutoSeededRandomPool &FixedSizeData<SIZE>::RandomPool() {
  static CryptoPP::AutoSeededRandomPool singleton;
  return singleton;
}

template<int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::CreateRandom() {
  FixedSizeData<SIZE> result;
  RandomPool().GenerateBlock(result._data, BINARY_LENGTH);
  return result;
}

template<int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::FromString(const std::string &data) {
  assert(data.size() == STRING_LENGTH);
  FixedSizeData<SIZE> result;
  CryptoPP::StringSource(data, true,
    new CryptoPP::HexDecoder(
      new CryptoPP::ArraySink(result._data, BINARY_LENGTH)
    )
  );
  return result;
}

template<int SIZE>
std::string FixedSizeData<SIZE>::ToString() const {
  std::string result;
  CryptoPP::ArraySource(_data, BINARY_LENGTH, true,
    new CryptoPP::HexEncoder(
      new CryptoPP::StringSink(result)
    )
  );
  assert(result.size() == STRING_LENGTH);
  return result;
}

template<int SIZE>
const unsigned char *FixedSizeData<SIZE>::data() const {
  return _data;
}

template<int SIZE>
void FixedSizeData<SIZE>::ToBinary(void *target) const {
  std::memcpy(target, _data, BINARY_LENGTH);
}

template<int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::FromBinary(const void *source) {
  FixedSizeData<SIZE> result;
  std::memcpy(result._data, source, BINARY_LENGTH);
  return result;
}

template<int SIZE>
bool operator==(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs) {
  return 0 == std::memcmp(lhs.data(), rhs.data(), FixedSizeData<SIZE>::BINARY_LENGTH);
}

template<int SIZE>
bool operator!=(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs) {
  return !operator==(lhs, rhs);
}

}

#endif
