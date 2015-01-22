#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>
#include "BlobStoreOnBlocks.h"

#include "BlobOnBlocks.h"

using std::unique_ptr;
using std::make_unique;

using blockstore::BlockStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {

using datanodestore::DataNodeStore;

BlobStoreOnBlocks::BlobStoreOnBlocks(unique_ptr<BlockStore> blockStore)
: _nodes(make_unique<DataNodeStore>(std::move(blockStore))) {
}

BlobStoreOnBlocks::~BlobStoreOnBlocks() {
}

unique_ptr<Blob> BlobStoreOnBlocks::create() {
  return make_unique<BlobOnBlocks>(_nodes->createNewLeafNode());
}

unique_ptr<Blob> BlobStoreOnBlocks::load(const Key &key) {
  return make_unique<BlobOnBlocks>(_nodes->load(key));
}

}
}
