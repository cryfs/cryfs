#include "BlobStoreWithRandomKeys.h"

#include "blobstore/utils/RandomKeyGenerator.h"

using namespace blobstore;

using std::string;

BlobWithKey BlobStoreWithRandomKeys::create(size_t size) {
  std::unique_ptr<BlobWithKey> result;
  do {
    result = create(_generateRandomKey(), size);
  } while (!result);

  return std::move(*result);
}

string BlobStoreWithRandomKeys::_generateRandomKey() {
  return RandomKeyGenerator::singleton().create();
}
