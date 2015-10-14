#pragma once
#ifndef MESSMER_CPPUTILS_DATA_FIXEDSIZEDATA_H_
#define MESSMER_CPPUTILS_DATA_FIXEDSIZEDATA_H_

#include <cryptopp/cryptopp/hex.h>
#include <cryptopp/cryptopp/osrng.h>
#include <string>
#include <cstring>
#include "../assert/assert.h"
#include "ThreadsafePseudoRandomPool.h"

namespace cpputils {

template<unsigned int SIZE>
class FixedSizeData {
public:
  //Non-virtual destructor because we want objects to be small
  ~FixedSizeData() {}

  static constexpr unsigned int BINARY_LENGTH = SIZE;
  static constexpr unsigned int STRING_LENGTH = 2 * BINARY_LENGTH; // Hex encoding

  static FixedSizeData<SIZE> CreatePseudoRandom();
  static FixedSizeData<SIZE> CreateOSRandom();
  //TODO Test Null()
  static FixedSizeData<SIZE> Null();

  static FixedSizeData<SIZE> FromString(const std::string &data);
  std::string ToString() const;

  static FixedSizeData<SIZE> FromBinary(const void *source);
  void ToBinary(void *target) const;

  const unsigned char *data() const;

private:
  FixedSizeData() {}
  static ThreadsafePseudoRandomPool &PseudoRandomPool();

  unsigned char _data[BINARY_LENGTH];
};

template<unsigned int SIZE> bool operator==(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs);
template<unsigned int SIZE> bool operator!=(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs);

// ----- Implementation -----

template<unsigned int SIZE> constexpr unsigned int FixedSizeData<SIZE>::BINARY_LENGTH;
template<unsigned int SIZE> constexpr unsigned int FixedSizeData<SIZE>::STRING_LENGTH;

template<unsigned int SIZE>
ThreadsafePseudoRandomPool &FixedSizeData<SIZE>::PseudoRandomPool() {
  static ThreadsafePseudoRandomPool singleton;
  return singleton;
}

template<unsigned int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::CreatePseudoRandom() {
  FixedSizeData<SIZE> result;
  PseudoRandomPool().GenerateBlock(result._data, BINARY_LENGTH);
  return result;
}

template<unsigned int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::CreateOSRandom() {
  FixedSizeData<SIZE> result;
  CryptoPP::OS_GenerateRandomBlock(true, result._data, BINARY_LENGTH);
  return result;
}

template<unsigned int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::Null() {
  FixedSizeData<SIZE> result;
  std::memset(result._data, 0, BINARY_LENGTH);
  return result;
}

template<unsigned int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::FromString(const std::string &data) {
  ASSERT(data.size() == STRING_LENGTH, "Wrong string size for parsing FixedSizeData");
  FixedSizeData<SIZE> result;
  CryptoPP::StringSource(data, true,
    new CryptoPP::HexDecoder(
      new CryptoPP::ArraySink(result._data, BINARY_LENGTH)
    )
  );
  return result;
}

template<unsigned int SIZE>
std::string FixedSizeData<SIZE>::ToString() const {
  std::string result;
  CryptoPP::ArraySource(_data, BINARY_LENGTH, true,
    new CryptoPP::HexEncoder(
      new CryptoPP::StringSink(result)
    )
  );
  ASSERT(result.size() == STRING_LENGTH, "Created wrongly sized string");
  return result;
}

template<unsigned int SIZE>
const unsigned char *FixedSizeData<SIZE>::data() const {
  return _data;
}

template<unsigned int SIZE>
void FixedSizeData<SIZE>::ToBinary(void *target) const {
  std::memcpy(target, _data, BINARY_LENGTH);
}

template<unsigned int SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::FromBinary(const void *source) {
  FixedSizeData<SIZE> result;
  std::memcpy(result._data, source, BINARY_LENGTH);
  return result;
}

template<unsigned int SIZE>
bool operator==(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs) {
  return 0 == std::memcmp(lhs.data(), rhs.data(), FixedSizeData<SIZE>::BINARY_LENGTH);
}

template<unsigned int SIZE>
bool operator!=(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs) {
  return !operator==(lhs, rhs);
}

}

#endif
