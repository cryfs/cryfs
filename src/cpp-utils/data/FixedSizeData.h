#pragma once
#ifndef MESSMER_CPPUTILS_DATA_FIXEDSIZEDATA_H_
#define MESSMER_CPPUTILS_DATA_FIXEDSIZEDATA_H_

#include <vendor_cryptopp/hex.h>
#include <string>
#include <cstring>
#include "../assert/assert.h"

namespace cpputils {

template<size_t SIZE>
class FixedSizeData final {
public:
  //Non-virtual destructor because we want objects to be small
  ~FixedSizeData() = default;

  static constexpr size_t BINARY_LENGTH = SIZE;
  static constexpr size_t STRING_LENGTH = 2 * BINARY_LENGTH; // Hex encoding

  //TODO Test Null()
  static FixedSizeData<SIZE> Null();

  static FixedSizeData<SIZE> FromString(const std::string &data);
  std::string ToString() const;

  static FixedSizeData<SIZE> FromBinary(const void *source);
  void ToBinary(void *target) const;

  const unsigned char *data() const;
  unsigned char *data();

  template<size_t size> FixedSizeData<size> take() const;
  template<size_t size> FixedSizeData<SIZE-size> drop() const;

private:
  FixedSizeData(): _data() {}
  template<size_t _SIZE> friend class FixedSizeData;

  unsigned char _data[BINARY_LENGTH];
};

template<size_t SIZE> bool operator==(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs);
template<size_t SIZE> bool operator!=(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs);

// ----- Implementation -----

template<size_t SIZE> constexpr size_t FixedSizeData<SIZE>::BINARY_LENGTH;
template<size_t SIZE> constexpr size_t FixedSizeData<SIZE>::STRING_LENGTH;

template<size_t SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::Null() {
  FixedSizeData<SIZE> result;
  std::memset(result._data, 0, BINARY_LENGTH);
  return result;
}

template<size_t SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::FromString(const std::string &data) {
  ASSERT(data.size() == STRING_LENGTH, "Wrong string size for parsing FixedSizeData");
  FixedSizeData<SIZE> result;
  {
    CryptoPP::StringSource _1(data, true,
      new CryptoPP::HexDecoder(
        new CryptoPP::ArraySink(result._data, BINARY_LENGTH)
      )
    );
  }
  return result;
}

template<size_t SIZE>
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

template<size_t SIZE>
const unsigned char *FixedSizeData<SIZE>::data() const {
  return _data;
}

template<size_t SIZE>
unsigned char *FixedSizeData<SIZE>::data() {
  return const_cast<unsigned char*>(const_cast<const FixedSizeData<SIZE>*>(this)->data());
}

template<size_t SIZE>
void FixedSizeData<SIZE>::ToBinary(void *target) const {
  std::memcpy(target, _data, BINARY_LENGTH);
}

template<size_t SIZE>
FixedSizeData<SIZE> FixedSizeData<SIZE>::FromBinary(const void *source) {
  FixedSizeData<SIZE> result;
  std::memcpy(result._data, source, BINARY_LENGTH);
  return result;
}

template<size_t SIZE> template<size_t size>
FixedSizeData<size> FixedSizeData<SIZE>::take() const {
  static_assert(size <= SIZE, "Out of bounds");
  FixedSizeData<size> result;
  std::memcpy(result._data, _data, size);
  return result;
}

template<size_t SIZE> template<size_t size>
FixedSizeData<SIZE-size> FixedSizeData<SIZE>::drop() const {
  static_assert(size <= SIZE, "Out of bounds");
  FixedSizeData<SIZE-size> result;
  std::memcpy(result._data, _data+size, SIZE-size);
  return result;
}

template<size_t SIZE>
bool operator==(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs) {
  return 0 == std::memcmp(lhs.data(), rhs.data(), FixedSizeData<SIZE>::BINARY_LENGTH);
}

template<size_t SIZE>
bool operator!=(const FixedSizeData<SIZE> &lhs, const FixedSizeData<SIZE> &rhs) {
  return !operator==(lhs, rhs);
}

}

#endif
