#include "BlockStoreWithRandomKeys.h"

using namespace blockstore;

using std::string;
using std::unique_ptr;

unique_ptr<Block> BlockStoreWithRandomKeys::create(size_t size) {
  auto result = tryCreate(size);
  while (!result) {
    result = tryCreate(size);
  }
  return result;
}

unique_ptr<Block> BlockStoreWithRandomKeys::tryCreate(size_t size) {
  Key key = Key::CreateRandom();
  return create(key, size);
}

