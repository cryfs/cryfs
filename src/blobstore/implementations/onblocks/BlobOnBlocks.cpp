#include <blobstore/implementations/onblocks/BlobOnBlocks.h>

using std::unique_ptr;
using blockstore::Block;

namespace blobstore {
namespace onblocks {

BlobOnBlocks::BlobOnBlocks(unique_ptr<Block> block)
: _block(std::move(block)) {

}

BlobOnBlocks::~BlobOnBlocks() {
}

void *BlobOnBlocks::data() {
  return const_cast<void*>(const_cast<const BlobOnBlocks*>(this)->data());
}

const void *BlobOnBlocks::data() const {
  return _block->data();
}

void BlobOnBlocks::flush() {
  _block->flush();
}

size_t BlobOnBlocks::size() const {
  return _block->size();
}

}
}
