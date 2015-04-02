#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGSTORE_H_

#include <mutex>
#include <memory>
#include <map>
#include <future>
#include <cassert>
#include <messmer/cpp-utils/macros.h>

template<class Resource, class CachedResourceRef, class Key>
class CachingStore {
public:
  CachingStore() {} //TODO Init member variables

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

  std::unique_ptr<Resource> add(std::unique_ptr<Resource> resource);
  std::unique_ptr<Resource> load(const Key &key);
  void remove(std::unique_ptr<Resource> block);

protected:
  //TODO Template instead of virtual for getKey?
  virtual const Key &getKey(const Resource &resource) const = 0;
  virtual std::unique_ptr<Resource> loadFromBaseStore(const Key &key) = 0;
  virtual void removeFromBaseStore(std::unique_ptr<Resource> resource) = 0;

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
  std::map<Key, OpenResource> _openResources;
  std::map<Key, std::promise<std::unique_ptr<Resource>>> _resourcesToRemove;

  std::unique_ptr<Resource> _add(std::unique_ptr<Resource> resource);
  std::unique_ptr<Resource> _createCachedResourceRef(Resource *resource, const Key &key);

  void release(const Key &key);
  friend class CachedResource;

  DISALLOW_COPY_AND_ASSIGN(CachingStore);
};

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<Resource> CachingStore<Resource, CachedResourceRef, Key>::add(std::unique_ptr<Resource> resource) {
  std::lock_guard<std::mutex> lock(_mutex);
  return _add(std::move(resource));
}

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<Resource> CachingStore<Resource, CachedResourceRef, Key>::_add(std::unique_ptr<Resource> resource) {
  auto insertResult = _openResources.emplace(getKey(*resource), std::move(resource));
  assert(true == insertResult.second);
  return _createCachedResourceRef(insertResult.first->second.getReference(), getKey(*resource));
}

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<Resource> CachingStore<Resource, CachedResourceRef, Key>::_createCachedResourceRef(Resource *resource, const Key &key) {
  auto resourceRef = std::make_unique<CachedResourceRef>(resource);
  resourceRef->init(this, getKey(*resource));
  return std::move(resourceRef);
}

template<class Resource, class CachedResourceRef, class Key>
std::unique_ptr<Resource> CachingStore<Resource, CachedResourceRef, Key>::load(const Key &key) {
  std::lock_guard<std::mutex> lock(_mutex);
  auto found = _openResources.find(key);
  if (found == _openResources.end()) {
	auto resource = loadFromBaseStore(key);
	if (resource.get() == nullptr) {
	  return nullptr;
	}
	return _add(std::move(resource));
  } else {
	return _createCachedResourceRef(found->second.getReference(), key);
  }
}

template<class Resource, class CachedResourceRef, class Key>
void CachingStore<Resource, CachedResourceRef, Key>::remove(std::unique_ptr<Resource> resource) {
  auto insertResult = _resourcesToRemove.emplace(getKey(*resource), std::promise<std::unique_ptr<Resource>>());
  assert(true == insertResult.second);
  resource.reset();

  //Wait for last resource user to release it
  auto resourceToRemove = insertResult.first->second.get_future().get();

  removeFromBaseStore(std::move(resourceToRemove));
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


#endif
