#include <blobstore/implementations/ondisk/Data.h>

#include <stdexcept>
#include <fstream>

using std::istream;
using std::ofstream;
using std::ifstream;
using std::ios;
using std::unique_ptr;
using std::make_unique;

namespace bf = boost::filesystem;

namespace blobstore {
namespace ondisk {

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

unique_ptr<Data> Data::LoadFromFile(const bf::path &filepath) {
  ifstream file(filepath.c_str(), ios::binary);
  size_t size = _getStreamSize(file);

  auto blob = make_unique<Data>(size);
  blob->_readFromStream(file);
  return blob;
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

} /* namespace ondisk */
} /* namespace blobstore */
