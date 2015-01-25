#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREE_H_

#include <memory>
#include "fspp/utils/macros.h"
#include "impl/GetLowestRightBorderNodeWithLessThanKChildrenOrNull.h"

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

  std::unique_ptr<datanodestore::DataLeafNode> addDataLeaf();
private:
  datanodestore::DataNodeStore *_nodeStore;
  std::unique_ptr<datanodestore::DataNode> _rootNode;

  std::unique_ptr<datanodestore::DataLeafNode> addDataLeafAt(datanodestore::DataInnerNode *insertPos);
  fspp::ptr::optional_ownership_ptr<datanodestore::DataNode> createChainOfInnerNodes(unsigned int num, datanodestore::DataLeafNode *leaf);
  std::unique_ptr<datanodestore::DataLeafNode> addDataLeafToFullTree();

  DISALLOW_COPY_AND_ASSIGN(DataTree);
};

}
}
}

#endif
