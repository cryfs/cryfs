#pragma once
#ifndef BLOCKSTORE_UTILS_BLOCKSTOREUTILS_H_
#define BLOCKSTORE_UTILS_BLOCKSTOREUTILS_H_

#include <memory>
#include "BlockWithKey.h"

namespace blockstore {
class BlockStore;
class Block;
namespace utils {

BlockWithKey copyToNewBlock(BlockStore *blockStore, const Block &block);

}
}

#endif
