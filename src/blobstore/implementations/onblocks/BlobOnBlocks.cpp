#include "parallelaccessdatatreestore/DataTreeRef.h"
#include "BlobOnBlocks.h"

#include "datanodestore/DataLeafNode.h"
#include "datanodestore/DataNodeStore.h"
#include "utils/Math.h"
#include <cmath>
#include <cpp-utils/assert/assert.h>
#include "datatreestore/LeafHandle.h"

using std::function;
using std::unique_lock;
using std::mutex;
using cpputils::unique_ref;
using cpputils::Data;
using blockstore::BlockId;
using blobstore::onblocks::datatreestore::LeafHandle;

namespace blobstore {
namespace onblocks {

using parallelaccessdatatreestore::DataTreeRef;

BlobOnBlocks::BlobOnBlocks(unique_ref<DataTreeRef> datatree)
: _datatree(std::move(datatree)), _sizeCache(boost::none), _mutex() {
}

BlobOnBlocks::~BlobOnBlocks() {
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

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

void BlobOnBlocks::_traverseLeaves(uint64_t beginByte, uint64_t sizeBytes, function<void (uint64_t leafOffset, LeafHandle leaf, uint32_t begin, uint32_t count)> onExistingLeaf, function<Data (uint64_t beginByte, uint32_t count)> onCreateLeaf) const {
  unique_lock<mutex> lock(_mutex); // TODO Multiple traverse calls in parallel?
  uint64_t endByte = beginByte + sizeBytes;
  uint64_t maxBytesPerLeaf = _datatree->maxBytesPerLeaf();
  uint32_t firstLeaf = beginByte / maxBytesPerLeaf;
  uint32_t endLeaf = utils::ceilDivision(endByte, maxBytesPerLeaf);
  bool blobIsGrowingFromThisTraversal = false;
  auto _onExistingLeaf = [&onExistingLeaf, beginByte, endByte, endLeaf, maxBytesPerLeaf, &blobIsGrowingFromThisTraversal] (uint32_t leafIndex, bool isRightBorderLeaf, LeafHandle leafHandle) {
      uint64_t indexOfFirstLeafByte = leafIndex * maxBytesPerLeaf;
      ASSERT(endByte > indexOfFirstLeafByte, "Traversal went too far right");
      uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
      uint32_t dataEnd = std::min(maxBytesPerLeaf, endByte - indexOfFirstLeafByte);
      // If we are traversing exactly until the last leaf, then the last leaf wasn't resized by the traversal and might have a wrong size. We have to fix it.
      if (isRightBorderLeaf) {
        ASSERT(leafIndex == endLeaf-1, "If we traversed further right, this wouldn't be the right border leaf.");
        auto leaf = leafHandle.node();
        if (leaf->numBytes() < dataEnd) {
          leaf->resize(dataEnd);
          blobIsGrowingFromThisTraversal = true;
        }
      }
      onExistingLeaf(indexOfFirstLeafByte, std::move(leafHandle), dataBegin, dataEnd-dataBegin);
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
  uint64_t realCount = std::max(INT64_C(0), std::min(static_cast<int64_t>(count), static_cast<int64_t>(size())-static_cast<int64_t>(offset)));
  _read(target, offset, realCount);
  return realCount;
}

void BlobOnBlocks::_read(void *target, uint64_t offset, uint64_t count) const {
  auto onExistingLeaf = [target, offset, count] (uint64_t indexOfFirstLeafByte, LeafHandle leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
      ASSERT(indexOfFirstLeafByte+leafDataOffset>=offset && indexOfFirstLeafByte-offset+leafDataOffset <= count && indexOfFirstLeafByte-offset+leafDataOffset+leafDataSize <= count, "Writing to target out of bounds");
      //TODO Simplify formula, make it easier to understand
      leaf.node()->read(static_cast<uint8_t*>(target) + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  };
  auto onCreateLeaf = [] (uint64_t /*beginByte*/, uint32_t /*count*/) -> Data {
      ASSERT(false, "Reading shouldn't create new leaves.");
  };
  _traverseLeaves(offset, count, onExistingLeaf, onCreateLeaf);
}

void BlobOnBlocks::write(const void *source, uint64_t offset, uint64_t count) {
  auto onExistingLeaf = [source, offset, count] (uint64_t indexOfFirstLeafByte, LeafHandle leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
      ASSERT(indexOfFirstLeafByte+leafDataOffset>=offset && indexOfFirstLeafByte-offset+leafDataOffset <= count && indexOfFirstLeafByte-offset+leafDataOffset+leafDataSize <= count, "Reading from source out of bounds");
      if (!leaf.isLoaded() && leafDataOffset == 0 && leafDataSize == leaf.nodeStore()->layout().maxBytesPerLeaf()) {
        // This is an optimization case - in case we write the full leaf and it isn't loaded yet, no need to load it, just overwrite it.
        Data leafData(leafDataSize);
        std::memcpy(leafData.data(), static_cast<const uint8_t*>(source) + indexOfFirstLeafByte - offset, leafDataSize);
        leaf.nodeStore()->overwriteLeaf(leaf.blockId(), std::move(leafData));
      } else {
        //TODO Simplify formula, make it easier to understand
        leaf.node()->write(static_cast<const uint8_t*>(source) + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset,
                           leafDataSize);
      }
  };
  auto onCreateLeaf = [source, offset, count] (uint64_t beginByte, uint32_t numBytes) -> Data {
      ASSERT(beginByte >= offset && beginByte-offset <= count && beginByte-offset+numBytes <= count, "Reading from source out of bounds");
      Data result(numBytes);
      //TODO Simplify formula, make it easier to understand
      std::memcpy(result.data(), static_cast<const uint8_t*>(source) + beginByte - offset, numBytes);
      return result;
  };
  _traverseLeaves(offset, count, onExistingLeaf, onCreateLeaf);
}

void BlobOnBlocks::flush() {
  _datatree->flush();
}

const BlockId &BlobOnBlocks::blockId() const {
  return _datatree->blockId();
}

unique_ref<DataTreeRef> BlobOnBlocks::releaseTree() {
  return std::move(_datatree);
}

}
}
