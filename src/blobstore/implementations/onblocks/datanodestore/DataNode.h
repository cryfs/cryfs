#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODE_H_

#include <blobstore/implementations/onblocks/datanodestore/DataNodeView.h>
#include "blockstore/utils/Data.h"

namespace blobstore {
namespace onblocks {
class DataNodeStore;

class DataNode {
public:
  virtual ~DataNode();

  virtual void read(off_t offset, size_t count, blockstore::Data *result) const = 0;
  virtual void write(off_t offset, size_t count, const blockstore::Data &data) = 0;

  virtual void resize(uint64_t newsize_bytes) = 0;
  virtual uint64_t numBytesInThisNode() const = 0;

  const Key &key() const;

  uint8_t depth() const;

protected:
  DataNode(DataNodeView block, const Key &key, DataNodeStore *nodestorage);

  DataNodeStore &storage();
  const DataNodeStore &storage() const;

  DataNodeView &node();
  const DataNodeView &node() const;

private:
  Key _key; //TODO Remove this and make blockstore::Block store the key
  DataNodeView _node;
  DataNodeStore *_nodestorage;

  DISALLOW_COPY_AND_ASSIGN(DataNode);
};

}
}


#endif
