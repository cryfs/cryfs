#include "DataTreeStore.h"
#include "../datanodestore/DataLeafNode.h"
#include "DataTree.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using boost::optional;
using boost::none;

using blobstore::onblocks::datanodestore::DataNodeStore;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTreeStore::DataTreeStore(unique_ref<DataNodeStore> nodeStore)
  : _nodeStore(std::move(nodeStore)) {
}

DataTreeStore::~DataTreeStore() {
}

optional<unique_ref<DataTree>> DataTreeStore::load(const blockstore::BlockId &blockId) {
  auto node = _nodeStore->load(blockId);
  if (node == none) {
    return none;
  }
  return make_unique_ref<DataTree>(_nodeStore.get(), std::move(*node));
}

unique_ref<DataTree> DataTreeStore::createNewTree() {
  auto newleaf = _nodeStore->createNewLeafNode(Data(0));
  return make_unique_ref<DataTree>(_nodeStore.get(), std::move(newleaf));
}

void DataTreeStore::remove(unique_ref<DataTree> tree) {
  _nodeStore->removeSubtree(tree->releaseRootNode());
}

void DataTreeStore::remove(const blockstore::BlockId &blockId) {
  auto tree = load(blockId);
  ASSERT(tree != none, "Tree to remove not found");
  remove(std::move(*tree));
}

}
}
}
