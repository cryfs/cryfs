#include "algorithms.h"
#include <cpp-utils/pointer/cast.h>
#include <blockstore/utils/Key.h>

#include "../../datanodestore/DataInnerNode.h"
#include "../../datanodestore/DataNodeStore.h"
#include <cpp-utils/assert/assert.h>

using std::function;
using cpputils::optional_ownership_ptr;
using cpputils::dynamic_pointer_move;
using cpputils::unique_ref;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blockstore::Key;
using boost::optional;
using boost::none;

namespace blobstore {
namespace onblocks {
namespace datatreestore {
namespace algorithms {

optional<unique_ref<DataInnerNode>> getLastChildAsInnerNode(DataNodeStore *nodeStore, const DataInnerNode &node) {
  Key key = node.LastChild()->key();
  auto lastChild = nodeStore->load(key);
  ASSERT(lastChild != none, "Couldn't load last child");
  return dynamic_pointer_move<DataInnerNode>(*lastChild);
}

//Returns the lowest right border node meeting the condition specified (exclusive the leaf).
//Returns nullptr, if no inner right border node meets the condition.
optional_ownership_ptr<DataInnerNode> GetLowestInnerRightBorderNodeWithConditionOrNull(DataNodeStore *nodeStore, datanodestore::DataNode *rootNode, function<bool(const DataInnerNode &)> condition) {
  optional_ownership_ptr<DataInnerNode> currentNode = cpputils::WithoutOwnership(dynamic_cast<DataInnerNode*>(rootNode));
  optional_ownership_ptr<DataInnerNode> result = cpputils::null<DataInnerNode>();
  for (unsigned int i=0; i < rootNode->depth(); ++i) {
    //TODO This unnecessarily loads the leaf node in the last loop run
    auto lastChild = getLastChildAsInnerNode(nodeStore, *currentNode);
    if (condition(*currentNode)) {
      result = std::move(currentNode);
    }
    ASSERT(lastChild != none || static_cast<int>(i) == rootNode->depth()-1, "Couldn't get last child as inner node but we're not deep enough yet for the last child to be a leaf");
    if (lastChild != none) {
      currentNode = cpputils::WithOwnership(std::move(*lastChild));
    }
  }

  return result;
}

optional_ownership_ptr<DataInnerNode> GetLowestRightBorderNodeWithMoreThanOneChildOrNull(DataNodeStore *nodeStore, DataNode *rootNode) {
  return GetLowestInnerRightBorderNodeWithConditionOrNull(nodeStore, rootNode, [] (const datanodestore::DataInnerNode &node) {
    return node.numChildren() > 1;
  });
}

optional_ownership_ptr<datanodestore::DataInnerNode> GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(datanodestore::DataNodeStore *nodeStore, datanodestore::DataNode *rootNode) {
  return GetLowestInnerRightBorderNodeWithConditionOrNull(nodeStore, rootNode, [] (const datanodestore::DataInnerNode &node) {
    return node.numChildren() < node.maxStoreableChildren();
  });
}

}
}
}
}
