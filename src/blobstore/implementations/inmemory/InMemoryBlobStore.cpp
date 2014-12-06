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
 : _blobs(), _generate_key_mutex() {}

BlobStore::BlobWithKey InMemoryBlobStore::create(size_t size) {
  std::string key = _generateKey();
  InMemoryBlob blob(size);
  _blobs.insert(make_pair(key, blob));

  //Return a copy of the stored InMemoryBlob
  return BlobWithKey(key, make_unique<InMemoryBlob>(blob));
}

string InMemoryBlobStore::_generateKey() {
  lock_guard<mutex> lock(_generate_key_mutex);

  string key;
  do {
    key = _generateRandomKey();
  } while (_blobs.count(key) > 0);

  return key;
}

string InMemoryBlobStore::_generateRandomKey() {
  return RandomKeyGenerator::singleton().create();
}

unique_ptr<Blob> InMemoryBlobStore::load(const string &key) {
  //Return a copy of the stored InMemoryBlob
  return make_unique<InMemoryBlob>(_blobs.at(key));
}

} /* namespace ondisk */
} /* namespace blobstore */
