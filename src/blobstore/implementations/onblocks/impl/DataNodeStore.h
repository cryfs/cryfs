#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATANODESTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATANODESTORE_H_

#include <memory>
#include "fspp/utils/macros.h"

namespace blockstore{
class Block;
class BlockStore;
class Key;
}

namespace blobstore {
namespace onblocks {
class DataNode;

class DataNodeStore {
public:
  DataNodeStore(std::unique_ptr<blockstore::BlockStore> blockstore);
  virtual ~DataNodeStore();

  static constexpr uint8_t MAX_DEPTH = 10;

  std::unique_ptr<DataNode> load(const blockstore::Key &key);
  std::unique_ptr<const DataNode> load(const blockstore::Key &key) const;

  std::unique_ptr<DataNode> createNewLeafNode();
  std::unique_ptr<DataNode> createNewInnerNode(const DataNode &first_child);

private:
  std::unique_ptr<DataNode> load(std::unique_ptr<blockstore::Block> block, const blockstore::Key &key);

  std::unique_ptr<blockstore::BlockStore> _blockstore;

  DISALLOW_COPY_AND_ASSIGN(DataNodeStore);
};

}
}

#endif
