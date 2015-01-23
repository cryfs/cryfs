#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_

#include <memory>
#include "fspp/utils/macros.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;
class DataInnerNode;
class DataLeafNode;
class DataNode;
}
namespace datatreestore {

class DataTree {
public:
  DataTree(datanodestore::DataNodeStore *nodeStore, std::unique_ptr<datanodestore::DataNode> rootNode);
  virtual ~DataTree();

  void addDataLeaf();
private:
  datanodestore::DataNodeStore *_nodeStore;
  std::unique_ptr<datanodestore::DataNode> _rootNode;

  std::unique_ptr<datanodestore::DataInnerNode> LowestRightBorderNodeWithLessThanKChildrenOrNull();
  std::unique_ptr<datanodestore::DataLeafNode> addDataLeafAt(datanodestore::DataInnerNode *insertPos);
  std::unique_ptr<datanodestore::DataInnerNode> createChainOfInnerNodes(unsigned int num, const datanodestore::DataLeafNode &leaf);
  std::unique_ptr<datanodestore::DataLeafNode> addDataLeafToFullTree();
  std::unique_ptr<datanodestore::DataNode> copyNode(const datanodestore::DataNode &source);

  DISALLOW_COPY_AND_ASSIGN(DataTree);
};

}
}
}

#endif
