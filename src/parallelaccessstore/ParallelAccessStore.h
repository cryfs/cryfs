#pragma once
#ifndef MESSMER_PARALLELACCESSSTORE_PARALLELACCESSSTORE_H_
#define MESSMER_PARALLELACCESSSTORE_PARALLELACCESSSTORE_H_

#include <mutex>
#include <memory>
#include <map>
#include <unordered_map>
#include <boost/thread/future.hpp>
#include <cassert>
#include <type_traits>
#include <cpp-utils/macros.h>
#include "ParallelAccessBaseStore.h"
#include <cpp-utils/assert/assert.h>


//TODO Refactor
//TODO Test cases

namespace parallelaccessstore {

template<class Resource, class ResourceRef, class Key>
class ParallelAccessStore final {
public:
  explicit ParallelAccessStore(cpputils::unique_ref<ParallelAccessBaseStore<Resource, Key>> baseStore);
    ~ParallelAccessStore() {
        ASSERT(_openResources.size() == 0, "Still resources open when trying to destruct");
        ASSERT(_resourcesToRemove.size() == 0, "Still resources to remove when trying to destruct");
    };

  class ResourceRefBase {
  public:
    //TODO Better way to initialize
    ResourceRefBase(): _parallelAccessStore(nullptr), _key(Key::Null()) {}
    void init(ParallelAccessStore *parallelAccessStore, const Key &key) {
      _parallelAccessStore = parallelAccessStore;
      _key = key;
    }
    virtual ~ResourceRefBase() {
      _parallelAccessStore->release(_key);
    }
  private:
    ParallelAccessStore *_parallelAccessStore;
    //TODO We're storing Key twice (here and in the base resource). Rather use getKey() on the base resource if possible somehow.
    Key _key;

    DISALLOW_COPY_AND_ASSIGN(ResourceRefBase);
  };

  bool isOpened(const Key &key) const;
  cpputils::unique_ref<ResourceRef> add(const Key &key, cpputils::unique_ref<Resource> resource);
  template<class ActualResourceRef>
  cpputils::unique_ref<ActualResourceRef> add(const Key &key, cpputils::unique_ref<Resource> resource, std::function<cpputils::unique_ref<ActualResourceRef>(Resource*)> createResourceRef);
  boost::optional<cpputils::unique_ref<ResourceRef>> load(const Key &key);
  boost::optional<cpputils::unique_ref<ResourceRef>> load(const Key &key, std::function<cpputils::unique_ref<ResourceRef>(Resource*)> createResourceRef);
  //loadOrAdd: If the resource is open, run onExists() on it. If not, run onAdd() and add the created resource. Then return the resource as if load() was called on it.
  cpputils::unique_ref<ResourceRef> loadOrAdd(const Key &key, std::function<void (ResourceRef*)> onExists, std::function<cpputils::unique_ref<Resource> ()> onAdd);
  cpputils::unique_ref<ResourceRef> loadOrAdd(const Key &key, std::function<void (ResourceRef*)> onExists, std::function<cpputils::unique_ref<Resource> ()> onAdd, std::function<cpputils::unique_ref<ResourceRef>(Resource*)> createResourceRef);
  void remove(const Key &key, cpputils::unique_ref<ResourceRef> block);
  void remove(const Key &key);

private:
  class OpenResource final {
  public:
	OpenResource(cpputils::unique_ref<Resource> resource): _resource(std::move(resource)), _refCount(0) {}
    OpenResource(OpenResource &&rhs) noexcept = default;

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

	cpputils::unique_ref<Resource> moveResourceOut() {
	  return std::move(_resource);
	}
  private:
	cpputils::unique_ref<Resource> _resource;
	uint32_t _refCount;

    DISALLOW_COPY_AND_ASSIGN(OpenResource);
  };

  mutable std::mutex _mutex;
  cpputils::unique_ref<ParallelAccessBaseStore<Resource, Key>> _baseStore;

  std::unordered_map<Key, OpenResource> _openResources;
  std::map<Key, boost::promise<cpputils::unique_ref<Resource>>> _resourcesToRemove;

  template<class ActualResourceRef>
  cpputils::unique_ref<ActualResourceRef> _add(const Key &key, cpputils::unique_ref<Resource> resource, std::function<cpputils::unique_ref<ActualResourceRef>(Resource*)> createResourceRef);

  boost::future<cpputils::unique_ref<Resource>> _resourceToRemoveFuture(const Key &key);
  cpputils::unique_ref<Resource> _waitForResourceToRemove(const Key &key, boost::future<cpputils::unique_ref<Resource>> resourceToRemoveFuture);

  void release(const Key &key);
  friend class CachedResource;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessStore);
};

template<class Resource, class ResourceRef, class Key>
ParallelAccessStore<Resource, ResourceRef, Key>::ParallelAccessStore(cpputils::unique_ref<ParallelAccessBaseStore<Resource, Key>> baseStore)
  : _mutex(),
  _baseStore(std::move(baseStore)),
  _openResources(),
  _resourcesToRemove() {
  static_assert(std::is_base_of<ResourceRefBase, ResourceRef>::value, "ResourceRef must inherit from ResourceRefBase");
}

template<class Resource, class ResourceRef, class Key>
bool ParallelAccessStore<Resource, ResourceRef, Key>::isOpened(const Key &key) const {
  std::lock_guard<std::mutex> lock(_mutex);
  return _openResources.find(key) != _openResources.end();
};

template<class Resource, class ResourceRef, class Key>
cpputils::unique_ref<ResourceRef> ParallelAccessStore<Resource, ResourceRef, Key>::add(const Key &key, cpputils::unique_ref<Resource> resource) {
  return add<ResourceRef>(key, std::move(resource), [] (Resource *resource) {
      return cpputils::make_unique_ref<ResourceRef>(resource);
  });
}

template<class Resource, class ResourceRef, class Key>
template<class ActualResourceRef>
cpputils::unique_ref<ActualResourceRef> ParallelAccessStore<Resource, ResourceRef, Key>::add(const Key &key, cpputils::unique_ref<Resource> resource, std::function<cpputils::unique_ref<ActualResourceRef>(Resource*)> createResourceRef) {
  static_assert(std::is_base_of<ResourceRef, ActualResourceRef>::value, "Wrong ResourceRef type");
  std::lock_guard<std::mutex> lock(_mutex);
  return _add<ActualResourceRef>(key, std::move(resource), createResourceRef);
}

template<class Resource, class ResourceRef, class Key>
template<class ActualResourceRef>
cpputils::unique_ref<ActualResourceRef> ParallelAccessStore<Resource, ResourceRef, Key>::_add(const Key &key, cpputils::unique_ref<Resource> resource, std::function<cpputils::unique_ref<ActualResourceRef>(Resource*)> createResourceRef) {
  static_assert(std::is_base_of<ResourceRef, ActualResourceRef>::value, "Wrong ResourceRef type");
  auto insertResult = _openResources.emplace(key, std::move(resource));
  ASSERT(true == insertResult.second, "Inserting failed. Already exists.");
  auto resourceRef = createResourceRef(insertResult.first->second.getReference());
  resourceRef->init(this, key);
  return resourceRef;
}

template<class Resource, class ResourceRef, class Key>
cpputils::unique_ref<ResourceRef> ParallelAccessStore<Resource, ResourceRef, Key>::loadOrAdd(const Key &key, std::function<void (ResourceRef*)> onExists, std::function<cpputils::unique_ref<Resource> ()> onAdd) {
    return loadOrAdd(key, onExists, onAdd, [] (Resource *res) {
        return cpputils::make_unique_ref<ResourceRef>(res);
    });
};

template<class Resource, class ResourceRef, class Key>
cpputils::unique_ref<ResourceRef> ParallelAccessStore<Resource, ResourceRef, Key>::loadOrAdd(const Key &key, std::function<void (ResourceRef*)> onExists, std::function<cpputils::unique_ref<Resource> ()> onAdd, std::function<cpputils::unique_ref<ResourceRef>(Resource*)> createResourceRef) {
    std::lock_guard<std::mutex> lock(_mutex);
    auto found = _openResources.find(key);
    if (found == _openResources.end()) {
        auto resource = onAdd();
        return _add(key, std::move(resource), createResourceRef);
    } else {
        auto resourceRef = createResourceRef(found->second.getReference());
        resourceRef->init(this, key);
        onExists(resourceRef.get());
        return resourceRef;
    }
};

template<class Resource, class ResourceRef, class Key>
boost::optional<cpputils::unique_ref<ResourceRef>> ParallelAccessStore<Resource, ResourceRef, Key>::load(const Key &key) {
  return load(key, [] (Resource *res) {
      return cpputils::make_unique_ref<ResourceRef>(res);
  });
};

template<class Resource, class ResourceRef, class Key>
boost::optional<cpputils::unique_ref<ResourceRef>> ParallelAccessStore<Resource, ResourceRef, Key>::load(const Key &key, std::function<cpputils::unique_ref<ResourceRef>(Resource*)> createResourceRef) {
  //TODO This lock doesn't allow loading different blocks in parallel. Can we only lock the requested key?
  std::lock_guard<std::mutex> lock(_mutex);
  auto found = _openResources.find(key);
  if (found == _openResources.end()) {
    auto resource = _baseStore->loadFromBaseStore(key);
    if (resource == boost::none) {
      return boost::none;
    }
  	return _add(key, std::move(*resource), createResourceRef);
  } else {
    auto resourceRef = createResourceRef(found->second.getReference());
    resourceRef->init(this, key);
    return resourceRef;
  }
}

template<class Resource, class ResourceRef, class Key>
void ParallelAccessStore<Resource, ResourceRef, Key>::remove(const Key &key, cpputils::unique_ref<ResourceRef> resource) {
  auto resourceToRemoveFuture = _resourceToRemoveFuture(key);

  cpputils::destruct(std::move(resource));

  //Wait for last resource user to release it
  auto resourceToRemove = resourceToRemoveFuture.get();
  std::lock_guard<std::mutex> lock(_mutex); // TODO Just added this as a precaution on a whim, but I seriously need to rethink locking here.
  _resourcesToRemove.erase(key); //TODO Is this erase causing a race condition?
  _baseStore->removeFromBaseStore(std::move(resourceToRemove));
}

template<class Resource, class ResourceRef, class Key>
boost::future<cpputils::unique_ref<Resource>> ParallelAccessStore<Resource, ResourceRef, Key>::_resourceToRemoveFuture(const Key &key) {
    std::lock_guard <std::mutex> lock(_mutex); // TODO Lock needed for _resourcesToRemove?
    auto insertResult = _resourcesToRemove.emplace(key, boost::promise<cpputils::unique_ref<Resource>>());
    ASSERT(true == insertResult.second, "Inserting failed");
    return insertResult.first->second.get_future();
};

template<class Resource, class ResourceRef, class Key>
void ParallelAccessStore<Resource, ResourceRef, Key>::remove(const Key &key) {
    auto found = _openResources.find(key);
    if (found != _openResources.end()) {
        auto resourceToRemoveFuture = _resourceToRemoveFuture(key);
        //Wait for last resource user to release it
        auto resourceToRemove = resourceToRemoveFuture.get();
        std::lock_guard<std::mutex> lock(_mutex); // TODO Just added this as a precaution on a whim, but I seriously need to rethink locking here.
        _resourcesToRemove.erase(key); //TODO Is this erase causing a race condition?
        _baseStore->removeFromBaseStore(std::move(resourceToRemove));
    } else {
        _baseStore->removeFromBaseStore(key);
    }
};

template<class Resource, class ResourceRef, class Key>
void ParallelAccessStore<Resource, ResourceRef, Key>::release(const Key &key) {
  std::lock_guard<std::mutex> lock(_mutex);
  auto found = _openResources.find(key);
  ASSERT(found != _openResources.end(), "Didn't find key");
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
