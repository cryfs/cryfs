#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_CACHINGDATATREESTORE_CACHINGDATATREESTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_CACHINGDATATREESTORE_CACHINGDATATREESTORE_H_

#include <memory>
#include <messmer/cpp-utils/macros.h>
#include <messmer/cachingstore/CachingStore.h>

namespace blockstore{
class Key;
}
namespace blobstore {
namespace onblocks {
namespace datatreestore {
class DataTreeStore;
class DataTree;
}
namespace cachingdatatreestore {
class CachedDataTreeRef;

//TODO Test CachingDataTreeStore

class CachingDataTreeStore {
public:
  CachingDataTreeStore(std::unique_ptr<datatreestore::DataTreeStore> dataTreeStore);
  virtual ~CachingDataTreeStore();

  std::unique_ptr<CachedDataTreeRef> load(const blockstore::Key &key);

  std::unique_ptr<CachedDataTreeRef> createNewTree();

  void remove(std::unique_ptr<CachedDataTreeRef> tree);

private:
  std::unique_ptr<datatreestore::DataTreeStore> _dataTreeStore;
  cachingstore::CachingStore<datatreestore::DataTree, CachedDataTreeRef, blockstore::Key> _cachingStore;

  DISALLOW_COPY_AND_ASSIGN(CachingDataTreeStore);
};

}
}
}

#endif
