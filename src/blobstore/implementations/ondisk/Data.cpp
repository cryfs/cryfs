#include <blobstore/implementations/ondisk/Data.h>

#include <stdexcept>

namespace blobstore {
namespace ondisk {

Data::Data(size_t size)
: _data(std::malloc(size)) {
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

} /* namespace ondisk */
} /* namespace blobstore */
