#include "BlobStoreWithRandomKeys.h"

#include "blobstore/utils/RandomKeyGenerator.h"

using namespace blobstore;

using std::string;
using std::lock_guard;
using std::mutex;

BlobStoreWithRandomKeys::BlobStoreWithRandomKeys()
  :_generate_key_mutex() {
}

BlobWithKey BlobStoreWithRandomKeys::create(size_t size) {
  return create(_generateKey(), size);
}

string BlobStoreWithRandomKeys::_generateKey() {
  lock_guard<mutex> lock(_generate_key_mutex);

  string key;
  do {
    key = _generateRandomKey();
  } while (exists(key));

  return key;
}

string BlobStoreWithRandomKeys::_generateRandomKey() {
  return RandomKeyGenerator::singleton().create();
}
