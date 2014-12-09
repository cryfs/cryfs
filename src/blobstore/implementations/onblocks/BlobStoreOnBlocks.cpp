#include "BlobStoreOnBlocks.h"

#include "BlobOnBlocks.h"

using std::unique_ptr;
using std::make_unique;

using blockstore::BlockStore;

namespace blobstore {
namespace onblocks {

BlobStoreOnBlocks::BlobStoreOnBlocks(unique_ptr<BlockStore> blockStore)
: _blocks(std::move(blockStore)) {
}

BlobStoreOnBlocks::~BlobStoreOnBlocks() {
}

BlobWithKey BlobStoreOnBlocks::create(size_t size) {
  auto block = _blocks->create(size);
  return BlobWithKey(block.key, make_unique<BlobOnBlocks>(std::move(block.block)));
}

unique_ptr<Blob> BlobStoreOnBlocks::load(const std::string &key) {
  return make_unique<BlobOnBlocks>(_blocks->load(key));
}

}
}
