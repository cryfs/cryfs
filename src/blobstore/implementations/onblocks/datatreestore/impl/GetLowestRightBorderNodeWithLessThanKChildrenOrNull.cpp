#include "GetLowestRightBorderNodeWithLessThanKChildrenOrNull.h"

#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"

#include "fspp/utils/pointer.h"

using std::unique_ptr;
using fspp::dynamic_pointer_move;
using fspp::ptr::optional_ownership_ptr;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datatreestore {
namespace impl {

unique_ptr<DataInnerNode> getLastChildAsInnerNode(DataNodeStore *nodeStore, const DataInnerNode &node) {
  Key key = node.LastChild()->key();
  auto lastChild = nodeStore->load(key);
  return dynamic_pointer_move<DataInnerNode>(lastChild);
}

optional_ownership_ptr<DataInnerNode> GetLowestRightBorderNodeWithLessThanKChildrenOrNull::run(DataNodeStore *nodeStore, DataNode *rootNode) {
  optional_ownership_ptr<DataInnerNode> currentNode = fspp::ptr::WithoutOwnership(dynamic_cast<DataInnerNode*>(rootNode));
  optional_ownership_ptr<DataInnerNode> result = fspp::ptr::null<DataInnerNode>();
  for (unsigned int i=0; i < rootNode->depth(); ++i) {
    auto lastChild = getLastChildAsInnerNode(nodeStore, *currentNode);
    if (currentNode->numChildren() < DataInnerNode::MAX_STORED_CHILDREN) {
      result = std::move(currentNode);
    }
    currentNode = std::move(lastChild);
  }

  return result;
}

}
}
}
}
