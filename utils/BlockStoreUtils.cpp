#include "../interface/BlockStore.h"
#include "BlockStoreUtils.h"
#include "Data.h"
#include <memory>
#include <cassert>

using std::unique_ptr;

namespace blockstore {
namespace utils {

unique_ptr<Block> copyToNewBlock(BlockStore *blockStore, const Block &block) {
  auto newBlock = blockStore->create(block.size());
  copyTo(newBlock.get(), block);
  return newBlock;
}

void copyTo(Block *target, const Block &source) {
  assert(target->size() == source.size());
  target->write(source.data(), 0, source.size());
}

void fillWithZeroes(Block *target) {
  Data zeroes(target->size());
  zeroes.FillWithZeroes();
  target->write(zeroes.data(), 0, target->size());
}

}
}
