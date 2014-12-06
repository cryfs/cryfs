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

unique_ptr<BlobWithKey> InMemoryBlobStore::create(const std::string &key, size_t size) {
  auto insert_result = _blobs.emplace(key, size);

  if (!insert_result.second) {
    return nullptr;
  }

  //Return a copy of the stored InMemoryBlob
  return make_unique<BlobWithKey>(key, make_unique<InMemoryBlob>(insert_result.first->second));
}

unique_ptr<Blob> InMemoryBlobStore::load(const string &key) {
  //Return a copy of the stored InMemoryBlob
  try {
    return make_unique<InMemoryBlob>(_blobs.at(key));
  } catch (const std::out_of_range &e) {
    return nullptr;
  }
}

} /* namespace ondisk */
} /* namespace blobstore */
