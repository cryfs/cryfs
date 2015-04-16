#include "DataTreeRef.h"
#include "ParallelAccessDataTreeStore.h"
#include "ParallelAccessDataTreeStoreAdapter.h"
#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataLeafNode.h"

using std::unique_ptr;
using std::make_unique;

using blobstore::onblocks::datatreestore::DataTreeStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
using datatreestore::DataTreeStore;
using datatreestore::DataTree;
namespace parallelaccessdatatreestore {

ParallelAccessDataTreeStore::ParallelAccessDataTreeStore(unique_ptr<DataTreeStore> dataTreeStore)
  : _dataTreeStore(std::move(dataTreeStore)), _parallelAccessStore(make_unique<ParallelAccessDataTreeStoreAdapter>(_dataTreeStore.get())) {
}

ParallelAccessDataTreeStore::~ParallelAccessDataTreeStore() {
}

unique_ptr<DataTreeRef> ParallelAccessDataTreeStore::load(const blockstore::Key &key) {
  return _parallelAccessStore.load(key);
}

unique_ptr<DataTreeRef> ParallelAccessDataTreeStore::createNewTree() {
  auto dataTree = _dataTreeStore->createNewTree();
  Key key = dataTree->key();
  return _parallelAccessStore.add(key, std::move(dataTree));
}

void ParallelAccessDataTreeStore::remove(unique_ptr<DataTreeRef> tree) {
  Key key = tree->key();
  return _parallelAccessStore.remove(key, std::move(tree));
}

}
}
}
