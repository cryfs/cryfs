#include "InMemoryBlobStore.h"
#include "InMemoryBlob.h"
#include "blobstore/utils/RandomKeyGenerator.h"

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;

namespace blobstore {
namespace inmemory {

InMemoryBlobStore::InMemoryBlobStore()
 : _blobs() {}

BlobWithKey InMemoryBlobStore::create(const std::string &key, size_t size) {
  InMemoryBlob blob(size);
  _blobs.insert(make_pair(key, blob));

  //Return a copy of the stored InMemoryBlob
  return BlobWithKey(key, make_unique<InMemoryBlob>(blob));
}

bool InMemoryBlobStore::exists(const std::string &key) {
  return _blobs.count(key) > 0;
}

unique_ptr<Blob> InMemoryBlobStore::load(const string &key) {
  //Return a copy of the stored InMemoryBlob
  return make_unique<InMemoryBlob>(_blobs.at(key));
}

} /* namespace ondisk */
} /* namespace blobstore */
