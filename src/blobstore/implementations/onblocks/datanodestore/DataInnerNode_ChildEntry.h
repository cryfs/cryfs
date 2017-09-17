#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_CHILDENTRY_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_CHILDENTRY_H_

#include <cpp-utils/macros.h>

namespace blobstore{
namespace onblocks{
namespace datanodestore{

struct DataInnerNode_ChildEntry final {
public:
  blockstore::BlockId blockId() const {
    return blockstore::BlockId::FromBinary(_blockIdData);
  }
private:
  void setBlockId(const blockstore::BlockId &blockId) {
    blockId.ToBinary(_blockIdData);
  }
  friend class DataInnerNode;
  uint8_t _blockIdData[blockstore::BlockId::BINARY_LENGTH];

  DISALLOW_COPY_AND_ASSIGN(DataInnerNode_ChildEntry);
};

}
}
}

#endif
