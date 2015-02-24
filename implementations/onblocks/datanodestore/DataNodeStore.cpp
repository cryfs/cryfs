#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNodeStore.h"
#include "messmer/blockstore/interface/BlockStore.h"
#include "messmer/blockstore/interface/Block.h"
#include "messmer/blockstore/utils/BlockStoreUtils.h"


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

unique_ptr<DataNode> DataNodeStore::load(unique_ptr<Block> block) {
  DataNodeView node(std::move(block));

  if (*node.Depth() == 0) {
    return unique_ptr<DataLeafNode>(new DataLeafNode(std::move(node)));
  } else if (*node.Depth() <= MAX_DEPTH) {
    return unique_ptr<DataInnerNode>(new DataInnerNode(std::move(node)));
  } else {
    throw runtime_error("Tree is to deep. Data corruption?");
  }
}

unique_ptr<DataInnerNode> DataNodeStore::createNewInnerNode(const DataNode &first_child) {
  auto block = _blockstore->create(DataNodeView::BLOCKSIZE_BYTES);
  return DataInnerNode::InitializeNewNode(std::move(block), first_child);
}

unique_ptr<DataLeafNode> DataNodeStore::createNewLeafNode() {
  auto block = _blockstore->create(DataNodeView::BLOCKSIZE_BYTES);
  return DataLeafNode::InitializeNewNode(std::move(block));
}

unique_ptr<DataNode> DataNodeStore::load(const Key &key) {
  auto block = _blockstore->load(key);
  if (block == nullptr) {
    return nullptr;
  }
  return load(std::move(block));
}

unique_ptr<DataNode> DataNodeStore::createNewNodeAsCopyFrom(const DataNode &source) {
  auto newBlock = blockstore::utils::copyToNewBlock(_blockstore.get(), source.node().block());
  return load(std::move(newBlock));
}

unique_ptr<DataNode> DataNodeStore::overwriteNodeWith(unique_ptr<DataNode> target, const DataNode &source) {
  Key key = target->key();
  {
    auto targetBlock = target->node().releaseBlock();
    target.reset();
    blockstore::utils::copyTo(targetBlock.get(), source.node().block());
  }
  return load(key);
}

void DataNodeStore::remove(unique_ptr<DataNode> node) {
  auto block = node->node().releaseBlock();
  node.reset();
  _blockstore->remove(std::move(block));
}

uint64_t DataNodeStore::numNodes() const {
  return _blockstore->numBlocks();
}

void DataNodeStore::removeSubtree(unique_ptr<DataNode> node) {
   DataInnerNode *inner = dynamic_cast<DataInnerNode*>(node.get());
   if (inner != nullptr) {
     for (int i = 0; i < inner->numChildren(); ++i) {
       auto child = load(inner->getChild(i)->key());
       removeSubtree(std::move(child));
     }
   }
   remove(std::move(node));
 }

}
}
}
