#include <blockstore/utils/Data.h>
#include <blockstore/utils/FileDoesntExistException.h>
#include "FileDoesntExistException.h"

#include <stdexcept>
#include <fstream>

using std::istream;
using std::ofstream;
using std::ifstream;
using std::ios;
using std::unique_ptr;
using std::make_unique;

namespace bf = boost::filesystem;

namespace blockstore {

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
}

Data::~Data() {
  std::free(_data);
  _data = nullptr;
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

void Data::FillWithZeroes() {
  std::memset(_data, 0, _size);
}

void Data::StoreToFile(const bf::path &filepath) const {
  ofstream file(filepath.c_str(), ios::binary | ios::trunc);
  file.write((const char*)_data, _size);
}

Data Data::LoadFromFile(const bf::path &filepath) {
  ifstream file(filepath.c_str(), ios::binary);
  _assertFileExists(file, filepath);
  size_t size = _getStreamSize(file);

  Data result(size);
  result._readFromStream(file);
  return result;
}

void Data::_assertFileExists(const ifstream &file, const bf::path &filepath) {
  if (!file.good()) {
    throw FileDoesntExistException(filepath);
  }
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

}
