#include "Data.h"
#include <stdexcept>
#include <fstream>

using std::istream;
using std::ofstream;
using std::ifstream;
using std::ios;

namespace bf = boost::filesystem;

namespace cpputils {

Data::Data(size_t size)
: _size(size), _data(std::malloc(size)) {
  if (nullptr == _data) {
    throw std::bad_alloc();
  }
}

Data::Data(Data &&rhs)
: _size(rhs._size), _data(rhs._data) {
  // Make rhs invalid, so the memory doesn't get freed in its destructor.
  rhs._data = nullptr;
  rhs._size = 0;
}

Data &Data::operator=(Data &&rhs) {
  std::free(_data);
  _data = rhs._data;
  _size = rhs._size;
  rhs._data = nullptr;
  rhs._size = 0;

  return *this;
}

Data::~Data() {
  std::free(_data);
  _data = nullptr;
}

Data Data::copy() const {
  Data copy(_size);
  std::memcpy(copy._data, _data, _size);
  return copy;
}

void *Data::data() {
  return const_cast<void*>(const_cast<const Data*>(this)->data());
}

const void *Data::data() const {
  return _data;
}

size_t Data::size() const {
  return _size;
}

Data &Data::FillWithZeroes() {
  std::memset(_data, 0, _size);
  return *this;
}

void Data::StoreToFile(const bf::path &filepath) const {
  ofstream file(filepath.c_str(), ios::binary | ios::trunc);
  file.write((const char*)_data, _size);
}

boost::optional<Data> Data::LoadFromFile(const bf::path &filepath) {
  ifstream file(filepath.c_str(), ios::binary);
  if (!file.good()) {
    return boost::none;
  }
  size_t size = _getStreamSize(file);

  Data result(size);
  result._readFromStream(file);
  return std::move(result);
}

size_t Data::_getStreamSize(istream &stream) {
  auto current_pos = stream.tellg();

  //Retrieve length
  stream.seekg(0, stream.end);
  auto endpos = stream.tellg();

  //Restore old position
  stream.seekg(current_pos, stream.beg);

  return endpos - current_pos;
}

void Data::_readFromStream(istream &stream) {
  stream.read((char*)_data, _size);
}

bool operator==(const Data &lhs, const Data &rhs) {
  return lhs.size() == rhs.size() && 0 == memcmp(lhs.data(), rhs.data(), lhs.size());
}

bool operator!=(const Data &lhs, const Data &rhs) {
  return !operator==(lhs, rhs);
}
}
