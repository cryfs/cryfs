#include "parallelaccessdatatreestore/DataTreeRef.h"
#include "BlobOnBlocks.h"

#include "datanodestore/DataLeafNode.h"
#include "utils/Math.h"
#include <cmath>
#include <cpp-utils/assert/assert.h>

using std::function;
using cpputils::unique_ref;
using cpputils::Data;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blockstore::Key;

namespace blobstore {
namespace onblocks {

using parallelaccessdatatreestore::DataTreeRef;

BlobOnBlocks::BlobOnBlocks(unique_ref<DataTreeRef> datatree)
: _datatree(std::move(datatree)), _sizeCache(boost::none) {
}

BlobOnBlocks::~BlobOnBlocks() {
}

uint64_t BlobOnBlocks::size() const {
  if (_sizeCache == boost::none) {
    _sizeCache = _datatree->numStoredBytes();
  }
  return *_sizeCache;
}

void BlobOnBlocks::resize(uint64_t numBytes) {
  _datatree->resizeNumBytes(numBytes);
  _sizeCache = numBytes;
}

void BlobOnBlocks::traverseLeaves(uint64_t beginByte, uint64_t sizeBytes, function<void (uint64_t, DataLeafNode *leaf, uint32_t, uint32_t)> func) const {
  uint64_t endByte = beginByte + sizeBytes;
  uint32_t firstLeaf = beginByte / _datatree->maxBytesPerLeaf();
  uint32_t endLeaf = utils::ceilDivision(endByte, _datatree->maxBytesPerLeaf());
  bool writingOutside = size() < endByte; // TODO Calling size() is slow because it has to traverse the tree
  _datatree->traverseLeaves(firstLeaf, endLeaf, [&func, beginByte, endByte, endLeaf, writingOutside](DataLeafNode *leaf, uint32_t leafIndex) {
    uint64_t indexOfFirstLeafByte = leafIndex * leaf->maxStoreableBytes();
    uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
    uint32_t dataEnd = std::min(leaf->maxStoreableBytes(), endByte - indexOfFirstLeafByte);
    if (leafIndex == endLeaf-1 && writingOutside) {
      // If we are traversing an area that didn't exist before, then the last leaf was just created with a wrong size. We have to fix it.
      leaf->resize(dataEnd);
    }
    func(indexOfFirstLeafByte, leaf, dataBegin, dataEnd-dataBegin);
  });
  if (writingOutside) {
    ASSERT(_datatree->numStoredBytes() == endByte, "Writing didn't grow by the correct number of bytes");
    _sizeCache = endByte;
  }
}

Data BlobOnBlocks::readAll() const {
  //TODO Querying size is inefficient. Is this possible without a call to size()?
  uint64_t count = size();
  Data result(count);
  _read(result.data(), 0, count);
  return result;
}

void BlobOnBlocks::read(void *target, uint64_t offset, uint64_t count) const {
  ASSERT(offset <= size() && offset + count <= size(), "BlobOnBlocks::read() read outside blob. Use BlobOnBlocks::tryRead() if this should be allowed.");
  uint64_t read = tryRead(target, offset, count);
  ASSERT(read == count, "BlobOnBlocks::read() couldn't read all requested bytes. Use BlobOnBlocks::tryRead() if this should be allowed.");
}

uint64_t BlobOnBlocks::tryRead(void *target, uint64_t offset, uint64_t count) const {
  //TODO Quite inefficient to call size() here, because that has to traverse the tree
  uint64_t realCount = std::max(UINT64_C(0), std::min(count, size()-offset));
  _read(target, offset, realCount);
  return realCount;
}

void BlobOnBlocks::_read(void *target, uint64_t offset, uint64_t count) const {
  traverseLeaves(offset, count, [target, offset] (uint64_t indexOfFirstLeafByte, const DataLeafNode *leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
      //TODO Simplify formula, make it easier to understand
      leaf->read((uint8_t*)target + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  });
}

void BlobOnBlocks::write(const void *source, uint64_t offset, uint64_t count) {
  traverseLeaves(offset, count, [source, offset] (uint64_t indexOfFirstLeafByte, DataLeafNode *leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
    //TODO Simplify formula, make it easier to understand
    leaf->write((uint8_t*)source + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  });
}

void BlobOnBlocks::flush() {
  _datatree->flush();
}

const Key &BlobOnBlocks::key() const {
  return _datatree->key();
}

unique_ref<DataTreeRef> BlobOnBlocks::releaseTree() {
  return std::move(_datatree);
}

}
}
