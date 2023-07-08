#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATALEAFNODE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATALEAFNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataInnerNode;

class DataLeafNode final: public DataNode {
public:
  static cpputils::unique_ref<DataLeafNode> CreateNewNode(blockstore::BlockStore *blockStore, const DataNodeLayout &layout, cpputils::Data data);
  static cpputils::unique_ref<DataLeafNode> OverwriteNode(blockstore::BlockStore *blockStore, const DataNodeLayout &layout, const blockstore::BlockId &blockId, cpputils::Data data);

  DataLeafNode(DataNodeView block);
  ~DataLeafNode() override;

  //Returning uint64_t, because calculations handling this probably need to be done in 64bit to support >4GB blobs.
  uint64_t maxStoreableBytes() const;

  void read(void *target, uint64_t offset, uint64_t size) const;
  void write(const void *source, uint64_t offset, uint64_t size);

  uint32_t numBytes() const;

  void resize(uint32_t size);

private:
  void fillDataWithZeroesFromTo(uint64_t begin, uint64_t end);

  DISALLOW_COPY_AND_ASSIGN(DataLeafNode);
};

}
}
}

#endif
