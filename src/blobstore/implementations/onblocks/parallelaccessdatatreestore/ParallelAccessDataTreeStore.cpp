#include "DataTreeRef.h"
#include "ParallelAccessDataTreeStore.h"
#include "ParallelAccessDataTreeStoreAdapter.h"
#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataLeafNode.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;

using blobstore::onblocks::datatreestore::DataTreeStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
using datatreestore::DataTreeStore;
using datatreestore::DataTree;
namespace parallelaccessdatatreestore {

//TODO Here and for other stores (DataTreeStore, ...): Make small functions inline

ParallelAccessDataTreeStore::ParallelAccessDataTreeStore(unique_ref<DataTreeStore> dataTreeStore)
  : _dataTreeStore(std::move(dataTreeStore)), _parallelAccessStore(make_unique_ref<ParallelAccessDataTreeStoreAdapter>(_dataTreeStore.get())) {
}

ParallelAccessDataTreeStore::~ParallelAccessDataTreeStore() {
}

optional<unique_ref<DataTreeRef>> ParallelAccessDataTreeStore::load(const blockstore::Key &key) {
  return _parallelAccessStore.load(key);
}

unique_ref<DataTreeRef> ParallelAccessDataTreeStore::createNewTree() {
  auto dataTree = _dataTreeStore->createNewTree();
  Key key = dataTree->key();
  return _parallelAccessStore.add(key, std::move(dataTree));
}

void ParallelAccessDataTreeStore::remove(unique_ref<DataTreeRef> tree) {
  Key key = tree->key();
  return _parallelAccessStore.remove(key, std::move(tree));
}



}
}
}
