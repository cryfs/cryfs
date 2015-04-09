#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGSTORE_H_

#include <mutex>
#include <memory>
#include <map>
#include <future>
#include <cassert>
#include <messmer/cpp-utils/macros.h>

#include "CachingBaseStore.h"

//TODO Refactor
//TODO Test cases

namespace cachingstore {

template<class Resource, class CachedResourceRef, class Key>
class CachingStore {
public:
  CachingStore(std::unique_ptr<CachingBaseStore<Resource, Key>> baseStore)
    : _mutex(),
	  _baseStore(std::move(baseStore)),
	  _openResources(),
	  _resourcesToRemove() {
  }

  //TODO Enforce CachedResourceRef inherits from CachedResource

  class CachedResource {
  public:
    //TODO Better way to initialize
    CachedResource(): _cachingStore(nullptr), _key(Key::CreateRandomKey()) {}
    void init(CachingStore *cachingStore, const Key &key) {
      _cachingStore = cachingStore;
      _key = key;
    }
    virtual ~CachedResource() {
      _cachingStore->release(_key);
    }
  private:
    CachingStore *_cachingStore;
    //TODO We're storing Key twice (here and in the base resource). Rather use getKey() on the base resource if possible somehow.
    Key _key;
  };

  std::unique_ptr<CachedResourceRef> add(const Key &key, std::unique_ptr<Resource> resource);
  std::unique_ptr<CachedResourceRef> load(const Key &key);
  void remove(const Key &key, std::unique_ptr<CachedResourceRef> block);

private:
  class OpenResource {
  public:
	OpenResource(std::unique_ptr<Resource> resource): _resource(std::move(resource)), _refCount(0) {}

	Resource *getReference() {
	  ++_refCount;
	  return _resource.get();
	}

	void releaseReference() {
	  --_refCount;
	}

	bool refCountIsZero() const {
	  return 0 == _refCount;
	}

	std::unique_ptr<Resource> moveResourceOut() {
	  return std::move(_resource);
	}
  private:
	std::unique_ptr<Resource> _resource;
	uint32_t _refCount;
  };

  std::mutex _mutex;
  std::unique_ptr<CachingBaseStore<Resource, Key>> _baseStore;

  std::map<Key, OpenResource> _openResources;
  std::map<Key, std::promise<std::unique_ptr<Resource>>> _resourcesToRemove;

  std::unique_ptr<CachedResourceRef> _add(const Key &key, std::unique_ptr<Resource> resource);
  std::unique_ptr<CachedResourceRef> _createCachedResourceRef(Resource *resource, const Key &key);

  void release(const Key &key);
  friend class CachedResource;

  DISALLOW_COPY_AND_ASSIGN(CachingStore);
};

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<CachedResourceRef> CachingStore<Resource, CachedResourceRef, Key>::add(const Key &key, std::unique_ptr<Resource> resource) {
  std::lock_guard<std::mutex> lock(_mutex);
  return _add(key, std::move(resource));
}

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<CachedResourceRef> CachingStore<Resource, CachedResourceRef, Key>::_add(const Key &key, std::unique_ptr<Resource> resource) {
  auto insertResult = _openResources.emplace(key, std::move(resource));
  assert(true == insertResult.second);
  return _createCachedResourceRef(insertResult.first->second.getReference(), key);
}

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<CachedResourceRef> CachingStore<Resource, CachedResourceRef, Key>::_createCachedResourceRef(Resource *resource, const Key &key) {
  auto resourceRef = std::make_unique<CachedResourceRef>(resource);
  resourceRef->init(this, key);
  return std::move(resourceRef);
}

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<CachedResourceRef> CachingStore<Resource, CachedResourceRef, Key>::load(const Key &key) {
  //TODO This lock doesn't allow loading different blocks in parallel. Can we do something with futures maybe?
  std::lock_guard<std::mutex> lock(_mutex);
  auto found = _openResources.find(key);
  if (found == _openResources.end()) {
	  auto resource = _baseStore->loadFromBaseStore(key);
	  if (resource.get() == nullptr) {
  	  return nullptr;
  	}
  	return _add(key, std::move(resource));
  } else {
	  return _createCachedResourceRef(found->second.getReference(), key);
  }
}

template<class Resource, class CachedResourceRef, class Key>
void CachingStore<Resource, CachedResourceRef, Key>::remove(const Key &key, std::unique_ptr<CachedResourceRef> resource) {
  auto insertResult = _resourcesToRemove.emplace(key, std::promise<std::unique_ptr<Resource>>());
  assert(true == insertResult.second);
  resource.reset();

  //Wait for last resource user to release it
  auto resourceToRemove = insertResult.first->second.get_future().get();

  _baseStore->removeFromBaseStore(std::move(resourceToRemove));
}

template<class Resource, class CachedResourceRef, class Key>
void CachingStore<Resource, CachedResourceRef, Key>::release(const Key &key) {
  std::lock_guard<std::mutex> lock(_mutex);
  auto found = _openResources.find(key);
  assert (found != _openResources.end());
  found->second.releaseReference();
  if (found->second.refCountIsZero()) {
	  auto foundToRemove = _resourcesToRemove.find(key);
	  if (foundToRemove != _resourcesToRemove.end()) {
	    foundToRemove->second.set_value(found->second.moveResourceOut());
	  }
	  _openResources.erase(found);
  }
}

}

#endif
