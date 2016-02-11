#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_IMPL_ALGORITHMS_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_IMPL_ALGORITHMS_H_

#include <cpp-utils/pointer/optional_ownership_ptr.h>

namespace blobstore {
namespace onblocks {
namespace datanodestore{
class DataNode;
class DataInnerNode;
class DataNodeStore;
}
namespace datatreestore {
namespace algorithms {

//Returns the lowest right border node with at least two children.
//Returns nullptr, if all right border nodes have only one child (since the root is a right border node, this means that the whole tree has exactly one leaf)
cpputils::optional_ownership_ptr<datanodestore::DataInnerNode> GetLowestRightBorderNodeWithMoreThanOneChildOrNull(datanodestore::DataNodeStore *nodeStore, datanodestore::DataNode *rootNode);

//Returns the lowest right border node with less than k children (not considering leaves).
//Returns nullptr, if all right border nodes have k children (the tree is full)
cpputils::optional_ownership_ptr<datanodestore::DataInnerNode> GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(datanodestore::DataNodeStore *nodeStore, datanodestore::DataNode *rootNode);

}
}
}
}

#endif
