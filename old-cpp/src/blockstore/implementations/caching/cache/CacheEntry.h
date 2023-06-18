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

  CacheEntry(CacheEntry&& rhs) noexcept: _lastAccess(std::move(rhs._lastAccess)), _value(std::move(rhs._value)) {}

  double ageSeconds() const {
    return static_cast<double>((currentTime() - _lastAccess).total_nanoseconds()) / static_cast<double>(1000000000);
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
