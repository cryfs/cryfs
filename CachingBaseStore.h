#ifndef MESSMER_CACHINGSTORE_CACHINGBASESTORE_H_
#define MESSMER_CACHINGSTORE_CACHINGBASESTORE_H_

#include <memory>

namespace cachingstore {

template<class Resource, class Key>
class CachingBaseStore {
public:
  virtual ~CachingBaseStore() {}
  virtual std::unique_ptr<Resource> loadFromBaseStore(const Key &key) = 0;
  virtual void removeFromBaseStore(std::unique_ptr<Resource> block) = 0;
};

}

#endif
