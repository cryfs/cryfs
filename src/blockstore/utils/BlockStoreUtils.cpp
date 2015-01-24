#include <blockstore/utils/BlockStoreUtils.h>
#include "blockstore/interface/BlockStore.h"

#include <memory>

using std::unique_ptr;

namespace blockstore {
namespace utils {

BlockWithKey copyToNewBlock(BlockStore *blockStore, const Block &block) {
  auto newBlock = blockStore->create(block.size());
  std::memcpy(newBlock.block->data(), block.data(), block.size());
  return newBlock;
}

}
}
