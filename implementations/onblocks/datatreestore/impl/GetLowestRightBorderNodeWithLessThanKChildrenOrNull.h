#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_IMPL_GETLOWESTRIGHTBORDERNODEWITHLESSTHANKCHILDRENORNULL_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_IMPL_GETLOWESTRIGHTBORDERNODEWITHLESSTHANKCHILDRENORNULL_H_

#include "messmer/cpp-utils/optional_ownership_ptr.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNode;
class DataInnerNode;
class DataNodeStore;
}
namespace datatreestore {
namespace impl {

class GetLowestRightBorderNodeWithLessThanKChildrenOrNull {
public:
  //Returns the lowest right border node with less than k children (not considering leaves).
  //Returns nullptr, if all right border nodes have k children (the tree is full)
  static cpputils::optional_ownership_ptr<datanodestore::DataInnerNode> run(datanodestore::DataNodeStore *nodeStore, datanodestore::DataNode *rootNode);
};

}
}
}
}

#endif
