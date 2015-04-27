#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEENTRY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEENTRY_H_

#include <ctime>
#include <memory>
#include <messmer/cpp-utils/macros.h>
#include <boost/date_time/posix_time/posix_time_types.hpp>

namespace blockstore {
class Block;
namespace caching {

class CacheEntry {
public:
  CacheEntry(std::unique_ptr<Block> block): _lastAccess(currentTime()), _block(std::move(block)) {
  }

  CacheEntry(CacheEntry &&) = default;

  double ageSeconds() const {
    return ((double)(currentTime() - _lastAccess).total_nanoseconds()) / ((double)1000000000);
  }

  std::unique_ptr<Block> releaseBlock() {
    return std::move(_block);
  }

private:
  boost::posix_time::ptime _lastAccess;
  std::unique_ptr<Block> _block;

  static boost::posix_time::ptime currentTime() {
	return boost::posix_time::microsec_clock::local_time();
  }

  DISALLOW_COPY_AND_ASSIGN(CacheEntry);
};

}
}


#endif
