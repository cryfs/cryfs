#include <blockstore/interface/helpers/BlockStoreWithRandomKeys.h>
#include <blockstore/utils/RandomKeyGenerator.h>

using namespace blockstore;

using std::string;

BlockWithKey BlockStoreWithRandomKeys::create(size_t size) {
  std::unique_ptr<BlockWithKey> result;
  do {
    result = create(_generateRandomKey(), size);
  } while (!result);

  return std::move(*result);
}

string BlockStoreWithRandomKeys::_generateRandomKey() {
  return RandomKeyGenerator::singleton().create();
}
