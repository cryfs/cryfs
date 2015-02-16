#include "GetLowestRightBorderNodeWithLessThanKChildrenOrNull.h"

#include "../../datanodestore/DataInnerNode.h"
#include "../../datanodestore/DataLeafNode.h"
#include "../../datanodestore/DataNodeStore.h"

#include "messmer/cpp-utils/pointer.h"

using std::unique_ptr;
using cpputils::dynamic_pointer_move;
using cpputils::optional_ownership_ptr;
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
  optional_ownership_ptr<DataInnerNode> currentNode = cpputils::WithoutOwnership(dynamic_cast<DataInnerNode*>(rootNode));
  optional_ownership_ptr<DataInnerNode> result = cpputils::null<DataInnerNode>();
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
