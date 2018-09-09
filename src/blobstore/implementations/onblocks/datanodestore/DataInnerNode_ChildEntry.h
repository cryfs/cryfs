#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_CHILDENTRY_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_CHILDENTRY_H_

#include <cpp-utils/macros.h>

namespace blobstore{
namespace onblocks{
namespace datanodestore{

struct DataInnerNode_ChildEntry final {
public:
  DataInnerNode_ChildEntry(const blockstore::BlockId &blockId): _blockId(blockId) {}

  const blockstore::BlockId& blockId() const {
    return _blockId;
  }

  DataInnerNode_ChildEntry(const DataInnerNode_ChildEntry&) = delete;
  DataInnerNode_ChildEntry& operator=(const DataInnerNode_ChildEntry&) = delete;
  DataInnerNode_ChildEntry(DataInnerNode_ChildEntry&&) = default;
  DataInnerNode_ChildEntry& operator=(DataInnerNode_ChildEntry&&) = default;

private:
  blockstore::BlockId _blockId;
};

}
}
}

#endif
