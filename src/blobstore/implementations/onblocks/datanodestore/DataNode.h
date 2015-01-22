#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODE_H_

#include "DataNodeView.h"
#include "blockstore/utils/Data.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;

class DataNode {
public:
  virtual ~DataNode();

  const blockstore::Key &key() const;

  uint8_t depth() const;

protected:
  DataNode(DataNodeView block, const blockstore::Key &key);

  DataNodeView &node();
  const DataNodeView &node() const;

private:
  blockstore::Key _key; //TODO Remove this and make blockstore::Block store the key
  DataNodeView _node;

  DISALLOW_COPY_AND_ASSIGN(DataNode);
};

}
}
}


#endif
