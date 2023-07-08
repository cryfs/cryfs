#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_CACHEDVALUE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_CACHEDVALUE_H_

#include <boost/optional.hpp>
#include <boost/thread/shared_mutex.hpp>
#include <functional>

namespace blobstore {
namespace onblocks {

// TODO Test
template<class T>
class CachedValue final {
public:
  CachedValue() :_cache(boost::none), _mutex() {}

  T getOrCompute(std::function<T ()> compute) {
    boost::upgrade_lock<boost::shared_mutex> readLock(_mutex);
    if (_cache == boost::none) {
      const boost::upgrade_to_unique_lock<boost::shared_mutex> writeLock(readLock);
      _cache = compute();
    }
    return *_cache;
  }

  void update(std::function<void (boost::optional<T>*)> func) {
    const boost::unique_lock<boost::shared_mutex> writeLock(_mutex);
    func(&_cache);
  }

  void clear() {
    update([] (boost::optional<T>* cache) {
      *cache = boost::none;
    });
  }

private:
  boost::optional<T> _cache;
  boost::shared_mutex _mutex;
};

}
}

#endif
