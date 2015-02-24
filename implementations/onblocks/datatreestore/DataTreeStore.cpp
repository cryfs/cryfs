#include "DataTreeStore.h"
#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataLeafNode.h"
#include "DataTree.h"

using std::unique_ptr;
using std::make_unique;

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTreeStore::DataTreeStore(unique_ptr<DataNodeStore> nodeStore)
  : _nodeStore(std::move(nodeStore)) {
}

DataTreeStore::~DataTreeStore() {
}

unique_ptr<DataTree> DataTreeStore::load(const blockstore::Key &key) {
  auto node = _nodeStore->load(key);
  if (node.get() == nullptr) {
    return nullptr;
  } else {
    return make_unique<DataTree>(_nodeStore.get(), std::move(node));
  }
}

unique_ptr<DataTree> DataTreeStore::createNewTree() {
  unique_ptr<DataNode> newleaf = _nodeStore->createNewLeafNode();
  return make_unique<DataTree>(_nodeStore.get(), std::move(newleaf));
}

void DataTreeStore::remove(unique_ptr<DataTree> tree) {
  auto root = tree->releaseRootNode();
  tree.reset();
  _nodeStore->removeSubtree(std::move(root));
}

}
}
}
