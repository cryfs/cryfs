#include <blobstore/implementations/onblocks/BlobOnBlocks.h>

using std::unique_ptr;
using blockstore::Block;

namespace blobstore {
namespace onblocks {

BlobOnBlocks::BlobOnBlocks(unique_ptr<Block> rootblock)
: _rootblock(std::move(rootblock)) {

}

BlobOnBlocks::~BlobOnBlocks() {
}

void *BlobOnBlocks::data() {
  return const_cast<void*>(const_cast<const BlobOnBlocks*>(this)->data());
}

const void *BlobOnBlocks::data() const {
  return _rootblock->data();
}

void BlobOnBlocks::flush() {
  _rootblock->flush();
}

size_t BlobOnBlocks::size() const {
  return _rootblock->size();
}

}
}
