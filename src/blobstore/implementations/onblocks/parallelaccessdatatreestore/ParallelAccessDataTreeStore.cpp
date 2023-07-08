#include "DataTreeRef.h"
#include "ParallelAccessDataTreeStore.h"
#include "ParallelAccessDataTreeStoreAdapter.h"
#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataLeafNode.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;

using blobstore::onblocks::datatreestore::DataTreeStore;
using blockstore::BlockId;

namespace blobstore {
namespace onblocks {
using datatreestore::DataTreeStore;
namespace parallelaccessdatatreestore {

//TODO Here and for other stores (DataTreeStore, ...): Make small functions inline

ParallelAccessDataTreeStore::ParallelAccessDataTreeStore(unique_ref<DataTreeStore> dataTreeStore)
  : _dataTreeStore(std::move(dataTreeStore)), _parallelAccessStore(make_unique_ref<ParallelAccessDataTreeStoreAdapter>(_dataTreeStore.get())) {
}

ParallelAccessDataTreeStore::~ParallelAccessDataTreeStore() {
}

optional<unique_ref<DataTreeRef>> ParallelAccessDataTreeStore::load(const blockstore::BlockId &blockId) {
  return _parallelAccessStore.load(blockId);
}

unique_ref<DataTreeRef> ParallelAccessDataTreeStore::createNewTree() {
  auto dataTree = _dataTreeStore->createNewTree();
  const BlockId blockId = dataTree->blockId();
  return _parallelAccessStore.add(blockId, std::move(dataTree));  // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
}

void ParallelAccessDataTreeStore::remove(unique_ref<DataTreeRef> tree) {
  const BlockId blockId = tree->blockId();
  return _parallelAccessStore.remove(blockId, std::move(tree));
}

void ParallelAccessDataTreeStore::remove(const BlockId &blockId) {
  return _parallelAccessStore.remove(blockId);
}


}
}
}
