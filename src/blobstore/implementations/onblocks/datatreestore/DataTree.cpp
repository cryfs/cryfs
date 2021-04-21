#include "DataTree.h"

#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataInnerNode.h"
#include "../datanodestore/DataLeafNode.h"
#include "../utils/Math.h"

#include "impl/algorithms.h"

#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include <cmath>
#include <cpp-utils/assert/assert.h>
#include "impl/LeafTraverser.h"
#include <boost/thread.hpp>
#include <blobstore/implementations/onblocks/utils/Math.h>

using blockstore::BlockId;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;

using std::function;
using boost::shared_mutex;
using boost::shared_lock;
using boost::unique_lock;
using boost::none;
using boost::optional;

using cpputils::optional_ownership_ptr;
using cpputils::unique_ref;
using cpputils::Data;
using namespace cpputils::logging;

//TODO shared_lock currently not enough for traverse because of root replacement. Can be fixed while keeping shared?

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTree::DataTree(DataNodeStore *nodeStore, unique_ref<DataNode> rootNode)
  : _treeStructureMutex(), _nodeStore(nodeStore), _rootNode(std::move(rootNode)), _blockId(_rootNode->blockId()), _sizeCache() {
}

DataTree::~DataTree() {
}

const BlockId &DataTree::blockId() const {
  return _blockId;
}

void DataTree::flush() const {
  // By grabbing a lock, we ensure that all modifying functions don't run currently and are therefore flushed.
  // It's only a shared lock, because this doesn't modify the tree structure.
  shared_lock<shared_mutex> lock(_treeStructureMutex);
  // We also have to flush the root node
  _rootNode->flush();
}

unique_ref<DataNode> DataTree::releaseRootNode() {
  // Lock also ensures that the root node is currently set (traversing unsets it temporarily)
  // It's a unique lock because this "modifies" tree structure by changing _rootNode.
  unique_lock<shared_mutex> lock(_treeStructureMutex);
  return std::move(_rootNode);
}

uint32_t DataTree::numNodes() const {
  uint32_t numNodesCurrentLevel = numLeaves();
  uint32_t totalNumNodes = numNodesCurrentLevel;
  for(size_t level = 0; level < _rootNode->depth(); ++level) {
    numNodesCurrentLevel = blobstore::onblocks::utils::ceilDivision(numNodesCurrentLevel, static_cast<uint32_t>(_nodeStore->layout().maxChildrenPerInnerNode()));
    totalNumNodes += numNodesCurrentLevel;
  }
  return totalNumNodes;
}

uint32_t DataTree::numLeaves() const {
  shared_lock<shared_mutex> lock(_treeStructureMutex);

  return _getOrComputeSizeCache().numLeaves;
}

uint64_t DataTree::numBytes() const {
  shared_lock<shared_mutex> lock(_treeStructureMutex);
  return _numBytes();
}

uint64_t DataTree::_numBytes() const {
  return _getOrComputeSizeCache().numBytes;
}

DataTree::SizeCache DataTree::_getOrComputeSizeCache() const {
  return _sizeCache.getOrCompute([this] () {
    return _computeSizeCache(*_rootNode);
  });
}

uint32_t DataTree::forceComputeNumLeaves() const {
  _sizeCache.clear();
  return numLeaves();
}

// NOLINTNEXTLINE(misc-no-recursion)
DataTree::SizeCache DataTree::_computeSizeCache(const DataNode &node) const {
  const DataLeafNode *leaf = dynamic_cast<const DataLeafNode*>(&node);
  if (leaf != nullptr) {
    return {1, leaf->numBytes()};
  }

  const DataInnerNode &inner = dynamic_cast<const DataInnerNode&>(node);
  uint32_t numLeavesInLeftChildren = static_cast<uint32_t>(inner.numChildren()-1) * _leavesPerFullChild(inner);
  uint64_t numBytesInLeftChildren = numLeavesInLeftChildren * _nodeStore->layout().maxBytesPerLeaf();
  auto lastChild = _nodeStore->load(inner.readLastChild().blockId());
  ASSERT(lastChild != none, "Couldn't load last child");
  SizeCache sizeInRightChild = _computeSizeCache(**lastChild);

  return SizeCache {
    numLeavesInLeftChildren + sizeInRightChild.numLeaves,
    numBytesInLeftChildren + sizeInRightChild.numBytes
  };
}

void DataTree::_traverseLeavesByLeafIndices(uint32_t beginIndex, uint32_t endIndex, bool readOnlyTraversal,
    function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf,
    function<Data (uint32_t index)> onCreateLeaf,
    function<void (DataInnerNode *node)> onBacktrackFromSubtree) const {
  if(endIndex <= beginIndex) {
    return;
  }

  // TODO no const cast
  LeafTraverser(_nodeStore, readOnlyTraversal).traverseAndUpdateRoot(&const_cast<DataTree*>(this)->_rootNode, beginIndex, endIndex, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
}

void DataTree::_traverseLeavesByByteIndices(uint64_t beginByte, uint64_t sizeBytes, bool readOnlyTraversal, function<void (uint64_t leafOffset, LeafHandle leaf, uint32_t begin, uint32_t count)> onExistingLeaf, function<Data (uint64_t beginByte, uint32_t count)> onCreateLeaf) const {
  if (sizeBytes == 0) {
    return;
  }

  uint64_t endByte = beginByte + sizeBytes;
  uint64_t _maxBytesPerLeaf = maxBytesPerLeaf();
  uint32_t firstLeaf = beginByte / _maxBytesPerLeaf;
  uint32_t endLeaf = utils::ceilDivision(endByte, _maxBytesPerLeaf);
  bool blobIsGrowingFromThisTraversal = false;
  auto _onExistingLeaf = [&onExistingLeaf, beginByte, endByte, endLeaf, _maxBytesPerLeaf, &blobIsGrowingFromThisTraversal] (uint32_t leafIndex, bool isRightBorderLeaf, LeafHandle leafHandle) {
    uint64_t indexOfFirstLeafByte = leafIndex * _maxBytesPerLeaf;
    ASSERT(endByte > indexOfFirstLeafByte, "Traversal went too far right");
    uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
    uint32_t dataEnd = std::min(_maxBytesPerLeaf, endByte - indexOfFirstLeafByte);
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
  auto _onCreateLeaf = [&onCreateLeaf, _maxBytesPerLeaf, beginByte, firstLeaf, endByte, endLeaf, &blobIsGrowingFromThisTraversal, readOnlyTraversal] (uint32_t leafIndex) -> Data {
    ASSERT(!readOnlyTraversal, "Cannot create leaves in a read-only traversal");
    blobIsGrowingFromThisTraversal = true;
    uint64_t indexOfFirstLeafByte = leafIndex * _maxBytesPerLeaf;
    ASSERT(endByte > indexOfFirstLeafByte, "Traversal went too far right");
    uint32_t dataBegin = utils::maxZeroSubtraction(beginByte, indexOfFirstLeafByte);
    uint32_t dataEnd = std::min(_maxBytesPerLeaf, endByte - indexOfFirstLeafByte);
    ASSERT(leafIndex == firstLeaf || dataBegin == 0, "Only the leftmost leaf can have a gap on the left.");
    ASSERT(leafIndex == endLeaf-1 || dataEnd == _maxBytesPerLeaf, "Only the rightmost leaf can have a gap on the right");
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
  auto _onBacktrackFromSubtree = [] (DataInnerNode* /*node*/) {};

  _traverseLeavesByLeafIndices(firstLeaf, endLeaf, readOnlyTraversal, _onExistingLeaf, _onCreateLeaf, _onBacktrackFromSubtree);

  ASSERT(!readOnlyTraversal || !blobIsGrowingFromThisTraversal, "Blob grew from traversal that didn't allow growing (i.e. reading)");

  if (blobIsGrowingFromThisTraversal) {
    _sizeCache.update([endLeaf, endByte] (optional<SizeCache>* cache) {
        *cache = SizeCache{endLeaf, endByte};
    });
  }
}

uint32_t DataTree::_leavesPerFullChild(const DataInnerNode &root) const {
  return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), static_cast<uint64_t>(root.depth())-1);
}

void DataTree::resizeNumBytes(uint64_t newNumBytes) {
  std::unique_lock<shared_mutex> lock(_treeStructureMutex);

  uint32_t newNumLeaves = std::max(UINT64_C(1), utils::ceilDivision(newNumBytes, _nodeStore->layout().maxBytesPerLeaf()));
  uint32_t newLastLeafSize = newNumBytes - (newNumLeaves-1) * _nodeStore->layout().maxBytesPerLeaf();
  uint32_t maxChildrenPerInnerNode = _nodeStore->layout().maxChildrenPerInnerNode();
  auto onExistingLeaf = [newLastLeafSize] (uint32_t /*index*/, bool /*isRightBorderLeaf*/, LeafHandle leafHandle) {
      auto leaf = leafHandle.node();
      // This is only called, if the new last leaf was already existing
      if (leaf->numBytes() != newLastLeafSize) {
        leaf->resize(newLastLeafSize);
      }
  };
  auto onCreateLeaf = [newLastLeafSize] (uint32_t /*index*/) -> Data {
      // This is only called, if the new last leaf was not existing yet
      return Data(newLastLeafSize).FillWithZeroes();
  };
  auto onBacktrackFromSubtree = [this, newNumLeaves, maxChildrenPerInnerNode] (DataInnerNode* node) {
      // This is only called for the right border nodes of the new tree.
      // When growing size, the following is a no-op. When shrinking, we're deleting the children that aren't needed anymore.
      uint32_t maxLeavesPerChild = utils::intPow(static_cast<uint64_t>(maxChildrenPerInnerNode), (static_cast<uint64_t>(node->depth())-1));
      uint32_t neededNodesOnChildLevel = utils::ceilDivision(newNumLeaves, maxLeavesPerChild);
      uint32_t neededSiblings = utils::ceilDivision(neededNodesOnChildLevel, maxChildrenPerInnerNode);
      uint32_t neededChildrenForRightBorderNode = neededNodesOnChildLevel - (neededSiblings-1) * maxChildrenPerInnerNode;
      ASSERT(neededChildrenForRightBorderNode <= node->numChildren(), "Node has too few children");
      // All children to the right of the new right-border-node are removed including their subtree.
      while(node->numChildren() > neededChildrenForRightBorderNode) {
        _nodeStore->removeSubtree(node->depth()-1, node->readLastChild().blockId());
        node->removeLastChild();
      }
  };

  _traverseLeavesByLeafIndices(newNumLeaves - 1, newNumLeaves, false, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
  _sizeCache.update([newNumLeaves, newNumBytes] (boost::optional<SizeCache>* cache) {
    *cache = SizeCache{newNumLeaves, newNumBytes};
  });

}

uint64_t DataTree::maxBytesPerLeaf() const {
  return _nodeStore->layout().maxBytesPerLeaf();
}

uint8_t DataTree::depth() const {
  shared_lock<shared_mutex> lock(_treeStructureMutex);
  return _rootNode->depth();
}

void DataTree::readBytes(void *target, uint64_t offset, uint64_t count) const {
  shared_lock<shared_mutex> lock(_treeStructureMutex);

  const uint64_t _size = _numBytes();
  if(offset > _size || offset + count > _size) {
    throw std::runtime_error("BlobOnBlocks::read() read outside blob. Use BlobOnBlocks::tryRead() if this should be allowed.");
  }
  const uint64_t read = _tryReadBytes(target, offset, count);
  if (read != count) {
    throw std::runtime_error("BlobOnBlocks::read() couldn't read all requested bytes. Use BlobOnBlocks::tryRead() if this should be allowed.");
  }
}

Data DataTree::readAllBytes() const {
  shared_lock<shared_mutex> lock(_treeStructureMutex);

  //TODO Querying numBytes can be inefficient. Is this possible without a call to size()?
  uint64_t count = _numBytes();
  Data result(count);
  _doReadBytes(result.data(), 0, count);

  return result;
}

uint64_t DataTree::tryReadBytes(void *target, uint64_t offset, uint64_t count) const {
  shared_lock<shared_mutex> lock(_treeStructureMutex);
  auto result = _tryReadBytes(target, offset, count);
  return result;
}

uint64_t DataTree::_tryReadBytes(void *target, uint64_t offset, uint64_t count) const {
  //TODO Quite inefficient to call size() here, because that has to traverse the tree
  const uint64_t _size = _numBytes();
  const uint64_t realCount = std::max(INT64_C(0), std::min(static_cast<int64_t>(count), static_cast<int64_t>(_size)-static_cast<int64_t>(offset)));
  _doReadBytes(target, offset, realCount);
  return realCount;
}

void DataTree::_doReadBytes(void *target, uint64_t offset, uint64_t count) const {
  auto onExistingLeaf = [target, offset, count] (uint64_t indexOfFirstLeafByte, LeafHandle leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
    ASSERT(indexOfFirstLeafByte+leafDataOffset>=offset && indexOfFirstLeafByte-offset+leafDataOffset <= count && indexOfFirstLeafByte-offset+leafDataOffset+leafDataSize <= count, "Writing to target out of bounds");
    //TODO Simplify formula, make it easier to understand
    leaf.node()->read(static_cast<uint8_t*>(target) + indexOfFirstLeafByte - offset + leafDataOffset, leafDataOffset, leafDataSize);
  };
  auto onCreateLeaf = [] (uint64_t /*beginByte*/, uint32_t /*count*/) -> Data {
    ASSERT(false, "Reading shouldn't create new leaves.");
  };

  _traverseLeavesByByteIndices(offset, count, true, onExistingLeaf, onCreateLeaf);
}

void DataTree::writeBytes(const void *source, uint64_t offset, uint64_t count) {
  unique_lock<shared_mutex> lock(_treeStructureMutex);

  auto onExistingLeaf = [source, offset, count] (uint64_t indexOfFirstLeafByte, LeafHandle leaf, uint32_t leafDataOffset, uint32_t leafDataSize) {
    ASSERT(indexOfFirstLeafByte+leafDataOffset>=offset && indexOfFirstLeafByte-offset+leafDataOffset <= count && indexOfFirstLeafByte-offset+leafDataOffset+leafDataSize <= count, "Reading from source out of bounds");
    if (leafDataOffset == 0 && leafDataSize == leaf.nodeStore()->layout().maxBytesPerLeaf()) {
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

  _traverseLeavesByByteIndices(offset, count, false, onExistingLeaf, onCreateLeaf);
}

}
}
}
