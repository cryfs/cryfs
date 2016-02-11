#include "DataTreeStore.h"
#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataLeafNode.h"
#include "DataTree.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTreeStore::DataTreeStore(unique_ref<DataNodeStore> nodeStore)
  : _nodeStore(std::move(nodeStore)) {
}

DataTreeStore::~DataTreeStore() {
}

optional<unique_ref<DataTree>> DataTreeStore::load(const blockstore::Key &key) {
  auto node = _nodeStore->load(key);
  if (node == none) {
    return none;
  }
  return make_unique_ref<DataTree>(_nodeStore.get(), std::move(*node));
}

unique_ref<DataTree> DataTreeStore::createNewTree() {
  auto newleaf = _nodeStore->createNewLeafNode();
  return make_unique_ref<DataTree>(_nodeStore.get(), std::move(newleaf));
}

void DataTreeStore::remove(unique_ref<DataTree> tree) {
  auto root = tree->releaseRootNode();
  cpputils::destruct(std::move(tree)); // Destruct tree
  _nodeStore->removeSubtree(std::move(root));
}

}
}
}
