#include <blockstore/interface/helpers/BlockStoreWithRandomKeys.h>

using namespace blockstore;

using std::string;

BlockWithKey BlockStoreWithRandomKeys::create(size_t size) {
  BlockWithKey result = tryCreate(size);
  while (!result.block) {
    result = tryCreate(size);
  }
  return result;
}

BlockWithKey BlockStoreWithRandomKeys::tryCreate(size_t size) {
  Key key = Key::CreateRandomKey();
  return BlockWithKey(key, create(key, size));
}

