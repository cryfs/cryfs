#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_

#include <memory>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include "../datanodestore/DataNodeView.h"
//TODO Replace with C++14 once std::shared_mutex is supported
#include <boost/thread/shared_mutex.hpp>
#include <blockstore/utils/Key.h>

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

  const blockstore::Key &key() const;
  //Returning uint64_t, because calculations handling this probably need to be done in 64bit to support >4GB blobs.
  uint64_t maxBytesPerLeaf() const;

  void traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (datanodestore::DataLeafNode*, uint32_t)> func);
  void resizeNumBytes(uint64_t newNumBytes);

  uint32_t numLeaves() const;
  uint64_t numStoredBytes() const;

  void flush() const;

private:
  mutable boost::shared_mutex _mutex;
  datanodestore::DataNodeStore *_nodeStore;
  cpputils::unique_ref<datanodestore::DataNode> _rootNode;

  cpputils::unique_ref<datanodestore::DataLeafNode> addDataLeaf();
  void removeLastDataLeaf();

  cpputils::unique_ref<datanodestore::DataNode> releaseRootNode();
  friend class DataTreeStore;

  cpputils::unique_ref<datanodestore::DataLeafNode> addDataLeafAt(datanodestore::DataInnerNode *insertPos);
  cpputils::optional_ownership_ptr<datanodestore::DataNode> createChainOfInnerNodes(unsigned int num, datanodestore::DataNode *child);
  cpputils::unique_ref<datanodestore::DataNode> createChainOfInnerNodes(unsigned int num, cpputils::unique_ref<datanodestore::DataNode> child);
  cpputils::unique_ref<datanodestore::DataLeafNode> addDataLeafToFullTree();

  void deleteLastChildSubtree(datanodestore::DataInnerNode *node);
  void ifRootHasOnlyOneChildReplaceRootWithItsChild();

  //TODO Use underscore for private methods
  void _traverseLeaves(datanodestore::DataNode *root, uint32_t leafOffset, uint32_t beginIndex, uint32_t endIndex, std::function<void (datanodestore::DataLeafNode*, uint32_t)> func);
  uint32_t leavesPerFullChild(const datanodestore::DataInnerNode &root) const;
  uint64_t _numStoredBytes() const;
  uint64_t _numStoredBytes(const datanodestore::DataNode &root) const;
  uint32_t _numLeaves(const datanodestore::DataNode &node) const;
  cpputils::optional_ownership_ptr<datanodestore::DataLeafNode> LastLeaf(datanodestore::DataNode *root);
  cpputils::unique_ref<datanodestore::DataLeafNode> LastLeaf(cpputils::unique_ref<datanodestore::DataNode> root);
  datanodestore::DataInnerNode* increaseTreeDepth(unsigned int levels);
  std::vector<cpputils::unique_ref<datanodestore::DataNode>> getOrCreateChildren(datanodestore::DataInnerNode *node, uint32_t begin, uint32_t end);
  cpputils::unique_ref<datanodestore::DataNode> addChildTo(datanodestore::DataInnerNode *node);

  DISALLOW_COPY_AND_ASSIGN(DataTree);
};

}
}
}

#endif
