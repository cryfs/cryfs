#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_

#include <memory>
#include <messmer/cpp-utils/macros.h>
#include <messmer/cpp-utils/optional_ownership_ptr.h>
#include "../datanodestore/DataNodeView.h"
//TODO Replace with C++14 once std::shared_mutex is supported
#include <boost/thread/shared_mutex.hpp>
#include <messmer/blockstore/utils/Key.h>

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
class DataTree {
public:
  DataTree(datanodestore::DataNodeStore *nodeStore, std::unique_ptr<datanodestore::DataNode> rootNode);
  virtual ~DataTree();

  const blockstore::Key &key() const;
  uint32_t maxBytesPerLeaf() const;

  void traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (datanodestore::DataLeafNode*, uint32_t)> func);
  void resizeNumBytes(uint64_t newNumBytes);

  uint32_t numLeaves() const;
  uint64_t numStoredBytes() const;

  void flush() const;

private:
  mutable boost::shared_mutex _mutex;
  datanodestore::DataNodeStore *_nodeStore;
  std::unique_ptr<datanodestore::DataNode> _rootNode;

  std::unique_ptr<datanodestore::DataLeafNode> addDataLeaf();
  void removeLastDataLeaf();

  std::unique_ptr<datanodestore::DataNode> releaseRootNode();
  friend class DataTreeStore;

  std::unique_ptr<datanodestore::DataLeafNode> addDataLeafAt(datanodestore::DataInnerNode *insertPos);
  cpputils::optional_ownership_ptr<datanodestore::DataNode> createChainOfInnerNodes(unsigned int num, datanodestore::DataNode *child);
  std::unique_ptr<datanodestore::DataNode> createChainOfInnerNodes(unsigned int num, std::unique_ptr<datanodestore::DataNode> child);
  std::unique_ptr<datanodestore::DataLeafNode> addDataLeafToFullTree();

  void deleteLastChildSubtree(datanodestore::DataInnerNode *node);
  void ifRootHasOnlyOneChildReplaceRootWithItsChild();

  //TODO Use underscore for private methods
  void _traverseLeaves(datanodestore::DataNode *root, uint32_t leafOffset, uint32_t beginIndex, uint32_t endIndex, std::function<void (datanodestore::DataLeafNode*, uint32_t)> func);
  uint32_t leavesPerFullChild(const datanodestore::DataInnerNode &root) const;
  uint64_t _numStoredBytes() const;
  uint64_t _numStoredBytes(const datanodestore::DataNode &root) const;
  cpputils::optional_ownership_ptr<datanodestore::DataLeafNode> LastLeaf(datanodestore::DataNode *root);
  std::unique_ptr<datanodestore::DataLeafNode> LastLeaf(std::unique_ptr<datanodestore::DataNode> root);
  datanodestore::DataInnerNode* increaseTreeDepth(unsigned int levels);
  std::vector<std::unique_ptr<datanodestore::DataNode>> getOrCreateChildren(datanodestore::DataInnerNode *node, uint32_t begin, uint32_t end);
  std::unique_ptr<datanodestore::DataNode> addChildTo(datanodestore::DataInnerNode *node);

  DISALLOW_COPY_AND_ASSIGN(DataTree);
};

}
}
}

#endif
