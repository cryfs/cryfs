#include "InMemoryBlob.h"
#include "InMemoryBlobStore.h"

#include <cstring>

using std::unique_ptr;
using std::make_unique;
using std::make_shared;
using std::istream;
using std::ostream;
using std::ifstream;
using std::ofstream;
using std::ios;

namespace blobstore {
namespace inmemory {

InMemoryBlob::InMemoryBlob(size_t size)
 : _data(make_shared<Data>(size)) {
}

InMemoryBlob::InMemoryBlob(const InMemoryBlob &rhs)
 : _data(rhs._data) {
}

InMemoryBlob::~InMemoryBlob() {
}

void *InMemoryBlob::data() {
  return _data->data();
}

const void *InMemoryBlob::data() const {
  return _data->data();
}

size_t InMemoryBlob::size() const {
  return _data->size();
}

void InMemoryBlob::flush() {
}

} /* namespace inmemory */
} /* namespace blobstore */
