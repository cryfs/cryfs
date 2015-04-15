#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHEENTRY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHEENTRY_H_

#include <ctime>
#include <memory>
#include <messmer/cpp-utils/macros.h>

namespace blockstore {
class Block;
namespace caching2 {

class CacheEntry {
public:
  CacheEntry(std::unique_ptr<Block> block): _lastAccess(time(nullptr)), _block(std::move(block)) {
  }

  double ageSeconds() {
    return difftime(time(nullptr), _lastAccess);
  }

  std::unique_ptr<Block> releaseBlock() {
    return std::move(_block);
  }

private:
  time_t _lastAccess;
  std::unique_ptr<Block> _block;

  DISALLOW_COPY_AND_ASSIGN(CacheEntry);
};

}
}


#endif
