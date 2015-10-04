#ifndef MESSMER_PARALLELACCESSSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSSTORE_H_
#define MESSMER_PARALLELACCESSSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSSTORE_H_

#include <mutex>
#include <memory>
#include <map>
#include <unordered_map>
#include <future>
#include <cassert>
#include <type_traits>
#include <messmer/cpp-utils/macros.h>
#include "ParallelAccessBaseStore.h"
#include <messmer/cpp-utils/assert/assert.h>


//TODO Refactor
//TODO Test cases

namespace parallelaccessstore {

template<class Resource, class ResourceRef, class Key>
class ParallelAccessStore {
public:
  explicit ParallelAccessStore(cpputils::unique_ref<ParallelAccessBaseStore<Resource, Key>> baseStore);

  class ResourceRefBase {
  public:
    //TODO Better way to initialize
    ResourceRefBase(): _cachingStore(nullptr), _key(Key::Null()) {}
    void init(ParallelAccessStore *cachingStore, const Key &key) {
      _cachingStore = cachingStore;
      _key = key;
    }
    virtual ~ResourceRefBase() {
      _cachingStore->release(_key);
    }
  private:
    ParallelAccessStore *_cachingStore;
    //TODO We're storing Key twice (here and in the base resource). Rather use getKey() on the base resource if possible somehow.
    Key _key;
  };

  cpputils::unique_ref<ResourceRef> add(const Key &key, cpputils::unique_ref<Resource> resource);
  template<class ActualResourceRef>
  cpputils::unique_ref<ActualResourceRef> add(const Key &key, cpputils::unique_ref<Resource> resource, std::function<cpputils::unique_ref<ActualResourceRef>(Resource*)> createResourceRef);
  boost::optional<cpputils::unique_ref<ResourceRef>> load(const Key &key);
  boost::optional<cpputils::unique_ref<ResourceRef>> load(const Key &key, std::function<cpputils::unique_ref<ResourceRef>(Resource*)> createResourceRef);
  void remove(const Key &key, cpputils::unique_ref<ResourceRef> block);

private:
  class OpenResource {
  public:
	OpenResource(cpputils::unique_ref<Resource> resource): _resource(std::move(resource)), _refCount(0) {}

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
  };

  std::mutex _mutex;
  cpputils::unique_ref<ParallelAccessBaseStore<Resource, Key>> _baseStore;

  std::unordered_map<Key, OpenResource> _openResources;
  std::map<Key, std::promise<cpputils::unique_ref<Resource>>> _resourcesToRemove;

  template<class ActualResourceRef>
  cpputils::unique_ref<ActualResourceRef> _add(const Key &key, cpputils::unique_ref<Resource> resource, std::function<cpputils::unique_ref<ActualResourceRef>(Resource*)> createResourceRef);

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
  ASSERT(true == insertResult.second, "Inserting failed");
  auto resourceRef = createResourceRef(insertResult.first->second.getReference());
  resourceRef->init(this, key);
  return resourceRef;
}

template<class Resource, class ResourceRef, class Key>
boost::optional<cpputils::unique_ref<ResourceRef>> ParallelAccessStore<Resource, ResourceRef, Key>::load(const Key &key) {
  return load(key, [] (Resource *res) {
      return cpputils::make_unique_ref<ResourceRef>(res);
  });
};

template<class Resource, class ResourceRef, class Key>
boost::optional<cpputils::unique_ref<ResourceRef>> ParallelAccessStore<Resource, ResourceRef, Key>::load(const Key &key, std::function<cpputils::unique_ref<ResourceRef>(Resource*)> createResourceRef) {
  //TODO This lock doesn't allow loading different blocks in parallel. Can we do something with futures maybe?
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
    return std::move(resourceRef);
  }
}

template<class Resource, class ResourceRef, class Key>
void ParallelAccessStore<Resource, ResourceRef, Key>::remove(const Key &key, cpputils::unique_ref<ResourceRef> resource) {
  auto insertResult = _resourcesToRemove.emplace(key, std::promise<cpputils::unique_ref<Resource>>());
  ASSERT(true == insertResult.second, "Inserting failed");
  cpputils::destruct(std::move(resource));

  //Wait for last resource user to release it
  auto resourceToRemove = insertResult.first->second.get_future().get();
  _resourcesToRemove.erase(key); //TODO Is this erase causing a race condition?

  _baseStore->removeFromBaseStore(std::move(resourceToRemove));
}

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
