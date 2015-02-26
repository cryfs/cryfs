#include "BlobOnBlocks.h"

#include "datatreestore/DataTree.h"
#include "datanodestore/DataLeafNode.h"
#include "utils/Math.h"
#include <cmath>

using std::unique_ptr;
using std::function;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blockstore::Key;

namespace blobstore {
namespace onblocks {

using datatreestore::DataTree;

BlobOnBlocks::BlobOnBlocks(unique_ptr<DataTree> datatree)
: _datatree(std::move(datatree)) {
}

BlobOnBlocks::~BlobOnBlocks() {
}

uint64_t BlobOnBlocks::size() const {
  return _datatree->numStoredBytes();
}

void BlobOnBlocks::resize(uint64_t numBytes) {
  _datatree->resizeNumBytes(numBytes);
}

void BlobOnBlocks::flush() const {
  _datatree->flush();
}

void BlobOnBlocks::traverseLeaves(uint64_t beginByte, uint64_t sizeBytes, function<void (uint64_t, void *, uint32_t)> func) const {
  uint64_t endByte = beginByte + sizeBytes;
  assert(endByte <= size());
  uint32_t firstLeaf = beginByte / _datatree->maxBytesPerLeaf();
  uint32_t endLeaf = utils::ceilDivision(endByte, _datatree->maxBytesPerLeaf());
  _datatree->traverseLeaves(firstLeaf, endLeaf, [&func, beginByte, endByte](DataLeafNode *leaf, uint32_t leafIndex) {
    uint64_t indexOfFirstLeafByte = leafIndex * leaf->maxStoreableBytes();
    uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
    uint32_t dataSize = std::min((uint64_t)leaf->maxStoreableBytes(), endByte - indexOfFirstLeafByte);
    func(indexOfFirstLeafByte, (uint8_t*)leaf->data() + dataBegin, dataSize);
  });
}

void BlobOnBlocks::read(void *target, uint64_t offset, uint64_t size) const {
  traverseLeaves(offset, size, [target] (uint64_t indexOfFirstLeafByte, void *leafDataBegin, uint32_t leafDataSize) {
    std::memcpy(target, leafDataBegin, leafDataSize);
  });
}

void BlobOnBlocks::write(const void *source, uint64_t offset, uint64_t size) {
  traverseLeaves(offset, size, [source] (uint64_t indexOfFirstLeafByte, void *leafDataBegin, uint32_t leafDataSize) {
    std::memcpy(leafDataBegin, source, leafDataSize);
  });
}

Key BlobOnBlocks::key() const {
  return _datatree->key();
}

}
}
