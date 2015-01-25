#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_IMPL_GETLOWESTRIGHTBORDERNODEWITHLESSTHANKCHILDRENORNULL_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_IMPL_GETLOWESTRIGHTBORDERNODEWITHLESSTHANKCHILDRENORNULL_H_

#include "fspp/utils/OptionalOwnershipPointer.h"

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
  static fspp::ptr::optional_ownership_ptr<datanodestore::DataInnerNode> run(datanodestore::DataNodeStore *nodeStore, datanodestore::DataNode *rootNode);
};

}
}
}
}

#endif
