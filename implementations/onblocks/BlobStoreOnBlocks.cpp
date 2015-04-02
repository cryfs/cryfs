#include <messmer/blockstore/implementations/caching/CachingBlockStore.h>
#include "datanodestore/DataLeafNode.h"
#include "datanodestore/DataNodeStore.h"
#include "datatreestore/DataTreeStore.h"
#include "datatreestore/DataTree.h"
#include "BlobStoreOnBlocks.h"
#include "BlobOnBlocks.h"
#include <messmer/cpp-utils/pointer.h>

using std::unique_ptr;
using std::make_unique;

using blockstore::BlockStore;
using blockstore::caching::CachingBlockStore;
using blockstore::Key;
using cpputils::dynamic_pointer_move;

namespace blobstore {
namespace onblocks {

using datanodestore::DataNodeStore;
using datatreestore::DataTreeStore;

BlobStoreOnBlocks::BlobStoreOnBlocks(unique_ptr<BlockStore> blockStore, uint32_t blocksizeBytes)
: _dataTreeStore(make_unique<DataTreeStore>(make_unique<DataNodeStore>(make_unique<CachingBlockStore>(std::move(blockStore)), blocksizeBytes))) {
}

BlobStoreOnBlocks::~BlobStoreOnBlocks() {
}

unique_ptr<Blob> BlobStoreOnBlocks::create() {
  return make_unique<BlobOnBlocks>(_dataTreeStore->createNewTree());
}

unique_ptr<Blob> BlobStoreOnBlocks::load(const Key &key) {
  auto tree = _dataTreeStore->load(key);
  if (tree == nullptr) {
  	return nullptr;
  }
  return make_unique<BlobOnBlocks>(std::move(tree));
}

void BlobStoreOnBlocks::remove(unique_ptr<Blob> blob) {
  auto _blob = dynamic_pointer_move<BlobOnBlocks>(blob);
  _dataTreeStore->remove(_blob->releaseTree());
}

}
}
