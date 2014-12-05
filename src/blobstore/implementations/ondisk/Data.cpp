#include <blobstore/implementations/ondisk/Data.h>

#include <stdexcept>
#include <fstream>

using std::ofstream;
using std::ios;

namespace blobstore {
namespace ondisk {

Data::Data(size_t size)
: _size(size), _data(std::malloc(size)) {
  if (nullptr == _data) {
    throw std::bad_alloc();
  }
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

void Data::StoreToFile(const boost::filesystem::path &filepath) const {
  ofstream file(filepath.c_str(), ios::binary | ios::trunc);
  file.write((const char*)_data, _size);
}

} /* namespace ondisk */
} /* namespace blobstore */
