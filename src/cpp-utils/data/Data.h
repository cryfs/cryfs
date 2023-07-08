#pragma once
#ifndef MESSMER_CPPUTILS_DATA_DATA_H_
#define MESSMER_CPPUTILS_DATA_DATA_H_

#include <cstdlib>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include "../macros.h"
#include <memory>
#include <fstream>
#include "../assert/assert.h"
#include "../pointer/unique_ref.h"

namespace cpputils {

struct Allocator {
  virtual ~Allocator() = default;

  virtual void* allocate(size_t size) = 0;
  virtual void free(void* ptr, size_t size) = 0;
};

class DefaultAllocator final : public Allocator {
public:
    void* allocate(size_t size) override {
      // std::malloc has implementation defined behavior for size=0.
      // Let's define the behavior.
      return std::malloc((size == 0) ? 1 : size);
    }

    void free(void* data, size_t /*size*/) override {
      std::free(data);
    }
};

class Data final {
public:
  explicit Data(size_t size, unique_ref<Allocator> allocator = make_unique_ref<DefaultAllocator>());
  ~Data();

  Data(Data &&rhs) noexcept;
  Data &operator=(Data &&rhs) noexcept;

  Data copy() const;

  //TODO Test copyAndRemovePrefix
  Data copyAndRemovePrefix(size_t prefixSize) const;

  void *data();
  const void *data() const;

  //TODO Test dataOffset
  void *dataOffset(size_t offset);
  const void *dataOffset(size_t offset) const;

  size_t size() const;

  Data &FillWithZeroes() &;
  Data &&FillWithZeroes() &&;

  void StoreToFile(const boost::filesystem::path &filepath) const;
  static boost::optional<Data> LoadFromFile(const boost::filesystem::path &filepath);

  //TODO Test LoadFromStream/StoreToStream
  static Data LoadFromStream(std::istream &stream);
  static Data LoadFromStream(std::istream &stream, size_t size);
  void StoreToStream(std::ostream &stream) const;

  // TODO Unify ToString/FromString functions from Data/FixedSizeData using free functions
  static Data FromString(const std::string &data, unique_ref<Allocator> allocator = make_unique_ref<DefaultAllocator>());
  std::string ToString() const;

private:
  std::unique_ptr<Allocator> _allocator;
  size_t _size;
  void *_data;

  static std::streampos _getStreamSize(std::istream &stream);
  void _readFromStream(std::istream &stream);
  void _free();

  DISALLOW_COPY_AND_ASSIGN(Data);
};

bool operator==(const Data &lhs, const Data &rhs);
bool operator!=(const Data &lhs, const Data &rhs);


// ---------------------------
// Inline function definitions
// ---------------------------

inline Data::Data(size_t size, unique_ref<Allocator> allocator)
        : _allocator(std::move(allocator)), _size(size), _data(_allocator->allocate(_size)) {
  if (nullptr == _data) {
    throw std::bad_alloc();
  }
}

inline Data::Data(Data &&rhs) noexcept
        : _allocator(std::move(rhs._allocator)), _size(rhs._size), _data(rhs._data) {
  // Make rhs invalid, so the memory doesn't get freed in its destructor.
  rhs._allocator = nullptr;
  rhs._data = nullptr;
  rhs._size = 0;
}

inline Data &Data::operator=(Data &&rhs) noexcept {
  _free();
  _allocator = std::move(rhs._allocator);
  _data = rhs._data;
  _size = rhs._size;
  rhs._allocator = nullptr;
  rhs._data = nullptr;
  rhs._size = 0;

  return *this;
}

inline Data::~Data() {
  _free();
}

inline void Data::_free() {
    if (nullptr != _allocator.get()) {
        _allocator->free(_data, _size);
    }
    _allocator = nullptr;
    _data = nullptr;
    _size = 0;
}

inline Data Data::copy() const {
  Data copy(_size);
  std::memcpy(copy._data, _data, _size);
  return copy;
}

inline Data Data::copyAndRemovePrefix(size_t prefixSize) const {
  ASSERT(prefixSize <= _size, "Can't remove more than there is");
  Data copy(_size - prefixSize);
  std::memcpy(copy.data(), dataOffset(prefixSize), copy.size());
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

inline Data &Data::FillWithZeroes() & {
    std::memset(_data, 0, _size);
    return *this;
}

inline Data &&Data::FillWithZeroes() && {
    return std::move(FillWithZeroes());
}

inline void Data::StoreToFile(const boost::filesystem::path &filepath) const {
  std::ofstream file(filepath.string().c_str(), std::ios::binary | std::ios::trunc);
  if (!file.good()) {
    throw std::runtime_error("Could not open file for writing");
  }
  StoreToStream(file);
  if (!file.good()) {
    throw std::runtime_error("Error writing to file");
  }
}

inline void Data::StoreToStream(std::ostream &stream) const {
  stream.write(static_cast<const char*>(_data), static_cast<std::streamsize>(_size));
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
