#pragma once
#ifndef MESSMER_BLOCKSTORE_UTILS_BLOCKSTOREUTILS_H_
#define MESSMER_BLOCKSTORE_UTILS_BLOCKSTOREUTILS_H_

#include <cpp-utils/pointer/unique_ref.h>

namespace blockstore {
class BlockStore;
class Block;
namespace utils {

cpputils::unique_ref<Block> copyToNewBlock(BlockStore *blockStore, const Block &block);
void copyTo(Block *target, const Block &source);
void fillWithZeroes(Block *target);

}
}

#endif
