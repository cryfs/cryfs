#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNodeStore.h"
#include "blockstore/interface/BlockStore.h"
#include "blockstore/interface/Block.h"


using blockstore::BlockStore;
using blockstore::Block;
using blockstore::Key;
using std::unique_ptr;
using std::make_unique;
using std::runtime_error;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataNodeStore::DataNodeStore(unique_ptr<BlockStore> blockstore)
: _blockstore(std::move(blockstore)) {
}

DataNodeStore::~DataNodeStore() {
}

unique_ptr<DataNode> DataNodeStore::load(unique_ptr<Block> block, const Key &key) {
  DataNodeView node(std::move(block));

  if (*node.Depth() == 0) {
    return unique_ptr<DataLeafNode>(new DataLeafNode(std::move(node), key));
  } else if (*node.Depth() <= MAX_DEPTH) {
    return unique_ptr<DataInnerNode>(new DataInnerNode(std::move(node), key));
  } else {
    throw runtime_error("Tree is to deep. Data corruption?");
  }
}

unique_ptr<DataInnerNode> DataNodeStore::createNewInnerNode(const DataNode &first_child) {
  auto block = _blockstore->create(DataNodeView::BLOCKSIZE_BYTES);
  auto newNode = make_unique<DataInnerNode>(std::move(block.block), block.key);
  newNode->InitializeNewNode(first_child);
  return std::move(newNode);
}

unique_ptr<DataLeafNode> DataNodeStore::createNewLeafNode() {
  auto block = _blockstore->create(DataNodeView::BLOCKSIZE_BYTES);
  auto newNode = make_unique<DataLeafNode>(std::move(block.block), block.key);
  newNode->InitializeNewNode();
  return std::move(newNode);
}

unique_ptr<DataNode> DataNodeStore::load(const Key &key) {
  return load(_blockstore->load(key), key);
}

unique_ptr<const DataNode> DataNodeStore::load(const Key &key) const {
  return const_cast<DataNodeStore*>(this)->load(key);
}

}
}
}
