#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATALEAFNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATALEAFNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {

class DataLeafNode: public DataNode {
public:
  DataLeafNode(DataNodeView block);
  virtual ~DataLeafNode();

  static constexpr uint32_t MAX_STORED_BYTES = DataNodeView::DATASIZE_BYTES;

  void InitializeNewNode();

  void read(off_t offset, size_t count, blockstore::Data *result) override;
  void write(off_t offset, size_t count, const blockstore::Data &data) override;

  uint64_t numBytesInThisNode() override;
  void resize(uint64_t newsize_bytes) override;

private:
  void fillDataWithZeroesFromTo(off_t begin, off_t end);
};

}
}

#endif
