#pragma once
#ifndef BLOCKSTORE_UTILS_BLOCKSTOREUTILS_H_
#define BLOCKSTORE_UTILS_BLOCKSTOREUTILS_H_

#include <memory>

namespace blockstore {
class BlockStore;
class Block;
namespace utils {

std::unique_ptr<Block> copyToNewBlock(BlockStore *blockStore, const Block &block);
void copyTo(Block *target, const Block &source);
void fillWithZeroes(Block *target);

}
}

#endif
