#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_CACHEENTRY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_CACHEENTRY_H_

#include <ctime>
#include <memory>
#include <cpp-utils/macros.h>
#include <boost/date_time/posix_time/posix_time_types.hpp>

namespace blockstore {
namespace caching {

template<class Key, class Value>
class CacheEntry final {
public:
  explicit CacheEntry(Value value): _lastAccess(currentTime()), _value(std::move(value)) {
  }

  CacheEntry(CacheEntry &&) = default;

  double ageSeconds() const {
    return ((double)(currentTime() - _lastAccess).total_nanoseconds()) / ((double)1000000000);
  }

  Value releaseValue() {
    return std::move(_value);
  }

private:
  boost::posix_time::ptime _lastAccess;
  Value _value;

  static boost::posix_time::ptime currentTime() {
	return boost::posix_time::microsec_clock::local_time();
  }

  DISALLOW_COPY_AND_ASSIGN(CacheEntry);
};

}
}


#endif
