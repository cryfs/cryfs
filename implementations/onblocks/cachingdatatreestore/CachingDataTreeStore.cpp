#include "CachingDataTreeStore.h"
#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataLeafNode.h"
#include "CachingDataTreeStoreAdapter.h"
#include "CachedDataTreeRef.h"

using std::unique_ptr;
using std::make_unique;

using blobstore::onblocks::datatreestore::DataTreeStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
using datatreestore::DataTreeStore;
using datatreestore::DataTree;
namespace cachingdatatreestore {

CachingDataTreeStore::CachingDataTreeStore(unique_ptr<DataTreeStore> dataTreeStore)
  : _dataTreeStore(std::move(dataTreeStore)), _cachingStore(make_unique<CachingDataTreeStoreAdapter>(_dataTreeStore.get())) {
}

CachingDataTreeStore::~CachingDataTreeStore() {
}

unique_ptr<CachedDataTreeRef> CachingDataTreeStore::load(const blockstore::Key &key) {
  return _cachingStore.load(key);
}

unique_ptr<CachedDataTreeRef> CachingDataTreeStore::createNewTree() {
  auto dataTree = _dataTreeStore->createNewTree();
  Key key = dataTree->key();
  return _cachingStore.add(key, std::move(dataTree));
}

void CachingDataTreeStore::remove(unique_ptr<CachedDataTreeRef> tree) {
  Key key = tree->key();
  return _cachingStore.remove(key, std::move(tree));
}

}
}
}
