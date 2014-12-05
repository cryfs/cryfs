#include "OnDiskBlob.h"

#include "OnDiskBlobStore.h"

#include <cstring>

using std::istream;
using std::ostream;

namespace blobstore {
namespace ondisk {

OnDiskBlob::OnDiskBlob(size_t size)
 : _size(size), _data(size) {
}

OnDiskBlob::~OnDiskBlob() {
}

void *OnDiskBlob::data() {
  return _data.data();
}

const void *OnDiskBlob::data() const {
  return _data.data();
}

size_t OnDiskBlob::size() const {
  return _size;
}

void OnDiskBlob::LoadDataFromStream(istream &stream) {
  stream.read((char*)_data.data(), _size);
}

void OnDiskBlob::StoreDataToStream(ostream &stream) const {
  stream.write((const char*)_data.data(), _size);
}

void OnDiskBlob::FillDataWithZeroes() {
  std::memset(_data.data(), 0, _size);
}

} /* namespace ondisk */
} /* namespace blobstore */
