#include "../interface/BlockStore.h"
#include "BlockStoreUtils.h"
#include <cpp-utils/data/Data.h>
#include <cassert>
#include <cpp-utils/assert/assert.h>

using cpputils::Data;
using cpputils::unique_ref;

namespace blockstore {
namespace utils {

unique_ref<Block> copyToNewBlock(BlockStore *blockStore, const Block &block) {
  Data data(block.size());
  std::memcpy(data.data(), block.data(), block.size());
  return blockStore->create(data);
}

void copyTo(Block *target, const Block &source) {
  ASSERT(target->size() == source.size(), "Can't copy block data when blocks have different sizes");
  target->write(source.data(), 0, source.size());
}

void fillWithZeroes(Block *target) {
  Data zeroes(target->size());
  zeroes.FillWithZeroes();
  target->write(zeroes.data(), 0, target->size());
}

}
}
