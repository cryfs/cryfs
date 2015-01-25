#include "gtest/gtest.h"

#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datatreestore/DataTree.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blockstore::testfake::FakeBlockStore;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

class DataTreeTest: public Test {
public:
  DataTreeTest():
    nodeStore(make_unique<FakeBlockStore>()) {
  }

  unique_ptr<DataTree> CreateLeafOnlyTree() {
    auto leafnode = nodeStore.createNewLeafNode();
    return make_unique<DataTree>(&nodeStore, std::move(leafnode));
  }

  DataNodeStore nodeStore;
};

}
}
}
