#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_

#include <memory>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include "../datanodestore/DataNodeView.h"
//TODO Replace with C++14 once std::shared_mutex is supported
#include <boost/thread/shared_mutex.hpp>
#include <blockstore/utils/BlockId.h>
#include "LeafHandle.h"
#include "impl/CachedValue.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;
class DataInnerNode;
class DataLeafNode;
class DataNode;
}
namespace datatreestore {

//TODO It is strange that DataLeafNode is still part in the public interface of DataTree. This should be separated somehow.
class DataTree final {
public:
  DataTree(datanodestore::DataNodeStore *nodeStore, cpputils::unique_ref<datanodestore::DataNode> rootNode);
  ~DataTree();

  const blockstore::BlockId &blockId() const;
  //Returning uint64_t, because calculations handling this probably need to be done in 64bit to support >4GB blobs.
  uint64_t maxBytesPerLeaf() const;

  uint64_t tryReadBytes(void *target, uint64_t offset, uint64_t count) const;
  void readBytes(void *target, uint64_t offset, uint64_t count) const;
  cpputils::Data readAllBytes() const;

  void writeBytes(const void *source, uint64_t offset, uint64_t count);

  void resizeNumBytes(uint64_t newNumBytes);

  uint32_t numNodes() const;
  uint32_t numLeaves() const;
  uint64_t numBytes() const;

  uint8_t depth() const;

  // only used by test cases
  uint32_t forceComputeNumLeaves() const;

  void flush() const;

private:
  // This mutex must protect the tree structure, i.e. which nodes exist and how they're connected.
  // Also protects total number of bytes (i.e. number of leaves + size of last leaf).
  // It also protects the data in leaf nodes, because writing bytes might grow the blob and change the structure.
  mutable boost::shared_mutex _treeStructureMutex;

  datanodestore::DataNodeStore *_nodeStore;
  cpputils::unique_ref<datanodestore::DataNode> _rootNode;
  blockstore::BlockId _blockId; // BlockId is stored in a member variable, since _rootNode is nullptr while traversing, but we still want to be able to return the blockId.

  struct SizeCache final {
    uint32_t numLeaves;
    uint64_t numBytes;
  };
  mutable CachedValue<SizeCache> _sizeCache;

  cpputils::unique_ref<datanodestore::DataNode> releaseRootNode();
  friend class DataTreeStore;

  void _traverseLeavesByLeafIndices(uint32_t beginIndex, uint32_t endIndex, bool readOnlyTraversal,
                                    std::function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf,
                                    std::function<cpputils::Data (uint32_t index)> onCreateLeaf,
                                    std::function<void (datanodestore::DataInnerNode *node)> onBacktrackFromSubtree) const;
  void _traverseLeavesByByteIndices(uint64_t beginByte, uint64_t sizeBytes, bool readOnlyTraversal, std::function<void (uint64_t leafOffset, LeafHandle leaf, uint32_t begin, uint32_t count)> onExistingLeaf, std::function<cpputils::Data (uint64_t beginByte, uint32_t count)> onCreateLeaf) const;

  uint32_t _leavesPerFullChild(const datanodestore::DataInnerNode &root) const;

  SizeCache _getOrComputeSizeCache() const;
  SizeCache _computeSizeCache(const datanodestore::DataNode &node) const;

  uint64_t _tryReadBytes(void *target, uint64_t offset, uint64_t count) const;
  void _doReadBytes(void *target, uint64_t offset, uint64_t count) const;
  uint64_t _numBytes() const;

  DISALLOW_COPY_AND_ASSIGN(DataTree);
};

}
}
}

#endif
