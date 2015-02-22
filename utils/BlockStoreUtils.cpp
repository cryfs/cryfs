#include <messmer/blockstore/interface/BlockStore.h>
#include <messmer/blockstore/utils/BlockStoreUtils.h>
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
  std::memcpy(target->data(), source.data(), source.size());
}

}
}
