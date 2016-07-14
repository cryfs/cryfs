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

void BlobOnBlocks::traverseLeaves(uint64_t beginByte, uint64_t sizeBytes, function<void (uint64_t leafOffset, DataLeafNode *leaf, uint32_t begin, uint32_t count)> onExistingLeaf, function<Data (uint64_t beginByte, uint32_t count)> onCreateLeaf) const {
  uint64_t endByte = beginByte + sizeBytes;
  uint64_t maxBytesPerLeaf = _datatree->maxBytesPerLeaf();
  uint32_t firstLeaf = beginByte / maxBytesPerLeaf;
  uint32_t endLeaf = utils::ceilDivision(endByte, maxBytesPerLeaf);
  bool blobIsGrowingFromThisTraversal = false;
  auto _onExistingLeaf = [&onExistingLeaf, beginByte, endByte, endLeaf, maxBytesPerLeaf, &blobIsGrowingFromThisTraversal] (uint32_t leafIndex, DataLeafNode *leaf) {
      uint64_t indexOfFirstLeafByte = leafIndex * maxBytesPerLeaf;
      ASSERT(endByte > indexOfFirstLeafByte, "Traversal went too far right");
      ASSERT(leafIndex == endLeaf-1 || leaf->numBytes() == maxBytesPerLeaf, "All leafes but the rightmost one have to have maximal size");
      uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
      uint32_t dataEnd = std::min(maxBytesPerLeaf, endByte - indexOfFirstLeafByte);
      if (leafIndex == endLeaf-1 && leaf->numBytes() < dataEnd) {
        // If we are traversing an area that didn't exist before (i.e. in the area of the last leaf that wasn't used before), then the last leaf might have a wrong size. We have to fix it.
        leaf->resize(dataEnd);
        blobIsGrowingFromThisTraversal = true;
      }
      onExistingLeaf(indexOfFirstLeafByte, leaf, dataBegin, dataEnd-dataBegin);
  };
  auto _onCreateLeaf = [&onCreateLeaf, maxBytesPerLeaf, beginByte, firstLeaf, endByte, endLeaf, &blobIsGrowingFromThisTraversal] (uint32_t leafIndex) -> Data {
      blobIsGrowingFromThisTraversal = true;
      uint64_t indexOfFirstLeafByte = leafIndex * maxBytesPerLeaf;
      ASSERT(endByte > indexOfFirstLeafByte, "Traversal went too far right");
      uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
      uint32_t dataEnd = std::min(maxBytesPerLeaf, endByte - indexOfFirstLeafByte);
      ASSERT(leafIndex == firstLeaf || dataBegin == 0, "Only the leftmost leaf can have a gap on the left.");
      ASSERT(leafIndex == endLeaf-1 || dataEnd == maxBytesPerLeaf, "Only the rightmost leaf can have a gap on the right");
      Data data = onCreateLeaf(indexOfFirstLeafByte + dataBegin, dataEnd-dataBegin);
      ASSERT(data.size() == dataEnd-dataBegin, "Returned leaf data with wrong size");
      // If this leaf is created but only partly in the traversed region (i.e. dataBegin > leafBegin), we have to fill the data before the traversed region with zeroes.
      if (dataBegin != 0) {
        Data actualData(dataBegin + data.size());
        std::memset(actualData.data(), 0, dataBegin);
        std::memcpy(actualData.dataOffset(dataBegin), data.data(), data.size());
        data = std::move(actualData);
      }
      return data;
  };
  _datatree->traverseLeaves(firstLeaf, endLeaf, _onExistingLeaf, _onCreateLeaf);
  if (blobIsGrowingFromThisTraversal) {
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
  uint64_t _size = size();
  ASSERT(offset <= _size && offset + count <= _size, "BlobOnBlocks::read() read outside blob. Use BlobOnBlocks::tryRead() if this should be allowed.");
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
  auto onExistingLeaf = [target, offset, count] (uint64_t indexOfFirstLeafByte, const DataLeafNode *leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
      ASSERT(indexOfFirstLeafByte+leafDataOffset>=offset && indexOfFirstLeafByte-offset+leafDataOffset <= count && indexOfFirstLeafByte-offset+leafDataOffset+leafDataSize <= count, "Writing to target out of bounds");
      //TODO Simplify formula, make it easier to understand
      leaf->read((uint8_t*)target + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  };
  auto onCreateLeaf = [] (uint64_t /*beginByte*/, uint32_t /*count*/) -> Data {
      ASSERT(false, "Reading shouldn't create new leaves.");
  };
  traverseLeaves(offset, count, onExistingLeaf, onCreateLeaf);
}

void BlobOnBlocks::write(const void *source, uint64_t offset, uint64_t count) {
  auto onExistingLeaf = [source, offset, count] (uint64_t indexOfFirstLeafByte, DataLeafNode *leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
      ASSERT(indexOfFirstLeafByte+leafDataOffset>=offset && indexOfFirstLeafByte-offset+leafDataOffset <= count && indexOfFirstLeafByte-offset+leafDataOffset+leafDataSize <= count, "Reading from source out of bounds");
      //TODO Simplify formula, make it easier to understand
      leaf->write((uint8_t*)source + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  };
  auto onCreateLeaf = [source, offset, count] (uint64_t beginByte, uint32_t numBytes) -> Data {
      ASSERT(beginByte >= offset && beginByte-offset <= count && beginByte-offset+numBytes <= count, "Reading from source out of bounds");
      Data result(numBytes);
      //TODO Simplify formula, make it easier to understand
      std::memcpy(result.data(), (uint8_t*)source + beginByte - offset, numBytes);
      return result;
  };
  traverseLeaves(offset, count, onExistingLeaf, onCreateLeaf);
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
