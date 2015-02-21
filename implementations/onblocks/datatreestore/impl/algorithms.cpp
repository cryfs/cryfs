#include "algorithms.h"
#include <messmer/cpp-utils/pointer.h>
#include <messmer/blockstore/utils/Key.h>

#include "../../datanodestore/DataInnerNode.h"
#include "../../datanodestore/DataNodeStore.h"

using std::function;
using std::unique_ptr;
using cpputils::optional_ownership_ptr;
using cpputils::dynamic_pointer_move;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datatreestore {
namespace algorithms {

unique_ptr<DataInnerNode> getLastChildAsInnerNode(DataNodeStore *nodeStore, const DataInnerNode &node) {
  Key key = node.LastChild()->key();
  auto lastChild = nodeStore->load(key);
  return dynamic_pointer_move<DataInnerNode>(lastChild);
}

//Returns the lowest right border node meeting the condition specified (exclusive the leaf).
//Returns nullptr, if no inner right border node meets the condition.
optional_ownership_ptr<DataInnerNode> GetLowestInnerRightBorderNodeWithConditionOrNull(DataNodeStore *nodeStore, datanodestore::DataNode *rootNode, function<bool(const DataInnerNode &)> condition) {
  optional_ownership_ptr<DataInnerNode> currentNode = cpputils::WithoutOwnership(dynamic_cast<DataInnerNode*>(rootNode));
  optional_ownership_ptr<DataInnerNode> result = cpputils::null<DataInnerNode>();
  for (unsigned int i=0; i < rootNode->depth(); ++i) {
    auto lastChild = getLastChildAsInnerNode(nodeStore, *currentNode);
    if (condition(*currentNode)) {
      result = std::move(currentNode);
    }
    currentNode = std::move(lastChild);
  }

  return result;
}

optional_ownership_ptr<DataInnerNode> GetLowestRightBorderNodeWithMoreThanOneChildOrNull(DataNodeStore *nodeStore, DataNode *rootNode) {
  return GetLowestInnerRightBorderNodeWithConditionOrNull(nodeStore, rootNode, [] (const datanodestore::DataInnerNode &node) {
    return node.numChildren() > 1;
  });
}
//TODO Test this

optional_ownership_ptr<datanodestore::DataInnerNode> GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(datanodestore::DataNodeStore *nodeStore, datanodestore::DataNode *rootNode) {
  return GetLowestInnerRightBorderNodeWithConditionOrNull(nodeStore, rootNode, [] (const datanodestore::DataInnerNode &node) {
    return node.numChildren() < DataInnerNode::MAX_STORED_CHILDREN;
  });
}

}
}
}
}
