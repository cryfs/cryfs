#include "parallelaccessdatatreestore/DataTreeRef.h"
#include "BlobOnBlocks.h"

#include "datanodestore/DataLeafNode.h"
#include "utils/Math.h"
#include <cmath>
#include <messmer/cpp-utils/assert/assert.h>

using std::function;
using cpputils::unique_ref;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blockstore::Key;

namespace blobstore {
namespace onblocks {

using parallelaccessdatatreestore::DataTreeRef;

BlobOnBlocks::BlobOnBlocks(unique_ref<DataTreeRef> datatree)
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

void BlobOnBlocks::traverseLeaves(uint64_t beginByte, uint64_t sizeBytes, function<void (uint64_t, DataLeafNode *leaf, uint32_t, uint32_t)> func) const {
  uint64_t endByte = beginByte + sizeBytes;
  uint32_t firstLeaf = beginByte / _datatree->maxBytesPerLeaf();
  uint32_t endLeaf = utils::ceilDivision(endByte, _datatree->maxBytesPerLeaf());
  bool traversingOutOfRange = _datatree->numStoredBytes() < endByte; //TODO numBytes() inefficient
  _datatree->traverseLeaves(firstLeaf, endLeaf, [&func, beginByte, endByte, endLeaf, traversingOutOfRange](DataLeafNode *leaf, uint32_t leafIndex) {
    uint64_t indexOfFirstLeafByte = leafIndex * leaf->maxStoreableBytes();
    uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
    uint32_t dataEnd = std::min((uint64_t)leaf->maxStoreableBytes(), endByte - indexOfFirstLeafByte);
    if (leafIndex == endLeaf-1 && traversingOutOfRange) {
      // If we are traversing an area that didn't exist before, then the last leaf was just created with a wrong size. We have to fix it.
      leaf->resize(dataEnd);
    }
    func(indexOfFirstLeafByte, leaf, dataBegin, dataEnd-dataBegin);
  });
}

void BlobOnBlocks::read(void *target, uint64_t offset, uint64_t count) const {
  ASSERT(offset <= _datatree->numStoredBytes() && offset + count <= size(), "BlobOnBlocks::read() read outside blob. Use BlobOnBlocks::tryRead() if this should be allowed.");
  uint64_t read = tryRead(target, offset, count);
  ASSERT(read == count, "BlobOnBlocks::read() couldn't read all requested bytes. Use BlobOnBlocks::tryRead() if this should be allowed.");
}

uint64_t BlobOnBlocks::tryRead(void *target, uint64_t offset, uint64_t count) const {
  //TODO Quite inefficient to call size() here, because that has to traverse the tree
  uint64_t realCount = std::max(UINT64_C(0), std::min(count, size()-offset));
  traverseLeaves(offset, realCount, [target, offset] (uint64_t indexOfFirstLeafByte, const DataLeafNode *leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
    //TODO Simplify formula, make it easier to understand
    leaf->read((uint8_t*)target + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  });
  return realCount;
}

void BlobOnBlocks::write(const void *source, uint64_t offset, uint64_t size) {
  traverseLeaves(offset, size, [source, offset] (uint64_t indexOfFirstLeafByte, DataLeafNode *leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
    //TODO Simplify formula, make it easier to understand
    leaf->write((uint8_t*)source + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  });
}

void BlobOnBlocks::flush() {
  _datatree->flush();
}

void BlobOnBlocks::resizeIfSmallerThan(uint64_t neededSize) {
  //TODO This is inefficient, because size() and resizeNumBytes() both traverse the tree. Better: _datatree->ensureMinSize(x)
  if (neededSize > size()) {
    _datatree->resizeNumBytes(neededSize);
  }
}

Key BlobOnBlocks::key() const {
  return _datatree->key();
}

unique_ref<DataTreeRef> BlobOnBlocks::releaseTree() {
  return std::move(_datatree);
}

}
}
