#include "parallelaccessdatatreestore/DataTreeRef.h"
#include "parallelaccessdatatreestore/ParallelAccessDataTreeStore.h"
#include <messmer/blockstore/implementations/parallelaccess/ParallelAccessBlockStore.h>
#include "datanodestore/DataLeafNode.h"
#include "datanodestore/DataNodeStore.h"
#include "datatreestore/DataTreeStore.h"
#include "datatreestore/DataTree.h"
#include "BlobStoreOnBlocks.h"
#include "BlobOnBlocks.h"
#include <messmer/cpp-utils/pointer/cast.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;

using blockstore::BlockStore;
using blockstore::parallelaccess::ParallelAccessBlockStore;
using blockstore::Key;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;

namespace blobstore {
namespace onblocks {

using datanodestore::DataNodeStore;
using datatreestore::DataTreeStore;
using parallelaccessdatatreestore::ParallelAccessDataTreeStore;

BlobStoreOnBlocks::BlobStoreOnBlocks(unique_ref<BlockStore> blockStore, uint32_t blocksizeBytes)
: _dataTreeStore(make_unique_ref<ParallelAccessDataTreeStore>(make_unique_ref<DataTreeStore>(make_unique_ref<DataNodeStore>(make_unique_ref<ParallelAccessBlockStore>(std::move(blockStore)), blocksizeBytes)))) {
}

BlobStoreOnBlocks::~BlobStoreOnBlocks() {
}

unique_ref<Blob> BlobStoreOnBlocks::create() {
  return make_unique_ref<BlobOnBlocks>(_dataTreeStore->createNewTree());
}

optional<unique_ref<Blob>> BlobStoreOnBlocks::load(const Key &key) {
  auto tree = _dataTreeStore->load(key);
  if (tree == none) {
  	return none;
  }
  return optional<unique_ref<Blob>>(make_unique_ref<BlobOnBlocks>(std::move(*tree)));
}

void BlobStoreOnBlocks::remove(unique_ref<Blob> blob) {
  auto _blob = dynamic_pointer_move<BlobOnBlocks>(blob);
  assert(_blob != none);
  _dataTreeStore->remove((*_blob)->releaseTree());
}

}
}
