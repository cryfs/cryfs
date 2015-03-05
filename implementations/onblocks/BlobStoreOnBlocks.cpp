#include "datanodestore/DataLeafNode.h"
#include "datanodestore/DataNodeStore.h"
#include "datatreestore/DataTreeStore.h"
#include "datatreestore/DataTree.h"
#include "BlobStoreOnBlocks.h"

#include "BlobOnBlocks.h"

using std::unique_ptr;
using std::make_unique;

using blockstore::BlockStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {

using datanodestore::DataNodeStore;
using datatreestore::DataTreeStore;

BlobStoreOnBlocks::BlobStoreOnBlocks(unique_ptr<BlockStore> blockStore, uint32_t blocksizeBytes)
: _dataTreeStore(make_unique<DataTreeStore>(make_unique<DataNodeStore>(std::move(blockStore), blocksizeBytes))) {
}

BlobStoreOnBlocks::~BlobStoreOnBlocks() {
}

unique_ptr<Blob> BlobStoreOnBlocks::create() {
  return make_unique<BlobOnBlocks>(_dataTreeStore->createNewTree());
}

unique_ptr<Blob> BlobStoreOnBlocks::load(const Key &key) {
  return make_unique<BlobOnBlocks>(_dataTreeStore->load(key));
}

}
}
