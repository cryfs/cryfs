#pragma once
#ifndef MESSMER_CPPUTILS_DATA_DATA_H_
#define MESSMER_CPPUTILS_DATA_DATA_H_

#include <cstdlib>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include "../macros.h"
#include <memory>
#include <fstream>

namespace cpputils {

class Data final {
public:
  explicit Data(size_t size);
  ~Data();

  Data(Data &&rhs); // move constructor
  Data &operator=(Data &&rhs); // move assignment

  Data copy() const;

  void *data();
  const void *data() const;

  //TODO Test dataOffset
  void *dataOffset(size_t offset);
  const void *dataOffset(size_t offset) const;

  size_t size() const;

  Data &FillWithZeroes();

  void StoreToFile(const boost::filesystem::path &filepath) const;
  static boost::optional<Data> LoadFromFile(const boost::filesystem::path &filepath);

  //TODO Test LoadFromStream/StoreToStream
  static Data LoadFromStream(std::istream &stream);
  static Data LoadFromStream(std::istream &stream, size_t size);
  void StoreToStream(std::ostream &stream) const;

private:
  size_t _size;
  void *_data;

  static std::streampos _getStreamSize(std::istream &stream);
  void _readFromStream(std::istream &stream);

  DISALLOW_COPY_AND_ASSIGN(Data);
};

bool operator==(const Data &lhs, const Data &rhs);
bool operator!=(const Data &lhs, const Data &rhs);


// ---------------------------
// Inline function definitions
// ---------------------------

inline Data::Data(size_t size)
        : _size(size), _data(std::malloc(size)) {
  if (nullptr == _data) {
    throw std::bad_alloc();
  }
}

inline Data::Data(Data &&rhs)
        : _size(rhs._size), _data(rhs._data) {
  // Make rhs invalid, so the memory doesn't get freed in its destructor.
  rhs._data = nullptr;
  rhs._size = 0;
}

inline Data &Data::operator=(Data &&rhs) {
  std::free(_data);
  _data = rhs._data;
  _size = rhs._size;
  rhs._data = nullptr;
  rhs._size = 0;

  return *this;
}

inline Data::~Data() {
  std::free(_data);
  _data = nullptr;
}

inline Data Data::copy() const {
  Data copy(_size);
  std::memcpy(copy._data, _data, _size);
  return copy;
}

inline void *Data::data() {
  return const_cast<void*>(const_cast<const Data*>(this)->data());
}

inline const void *Data::data() const {
  return _data;
}

inline void *Data::dataOffset(size_t offset) {
  return const_cast<void*>(const_cast<const Data*>(this)->dataOffset(offset));
}

inline const void *Data::dataOffset(size_t offset) const {
  return static_cast<const uint8_t*>(data()) + offset;
}

inline size_t Data::size() const {
  return _size;
}

inline Data &Data::FillWithZeroes() {
  std::memset(_data, 0, _size);
  return *this;
}

inline void Data::StoreToFile(const boost::filesystem::path &filepath) const {
  std::ofstream file(filepath.c_str(), std::ios::binary | std::ios::trunc);
  if (!file.good()) {
    throw std::runtime_error("Could not open file for writing");
  }
  StoreToStream(file);
}

inline void Data::StoreToStream(std::ostream &stream) const {
  stream.write(static_cast<const char*>(_data), _size);
}

inline Data Data::LoadFromStream(std::istream &stream) {
  return LoadFromStream(stream, _getStreamSize(stream));
}

inline bool operator==(const Data &lhs, const Data &rhs) {
  return lhs.size() == rhs.size() && 0 == memcmp(lhs.data(), rhs.data(), lhs.size());
}

inline bool operator!=(const Data &lhs, const Data &rhs) {
  return !operator==(lhs, rhs);
}

}

#endif
