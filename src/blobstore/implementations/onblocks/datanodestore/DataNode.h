#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODE_H_

#include "DataNodeView.h"
#include <cpp-utils/data/Data.h>

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;
class DataInnerNode;

class DataNode {
public:
  virtual ~DataNode();

  const blockstore::BlockId &blockId() const;

  uint8_t depth() const;

  static cpputils::unique_ref<DataInnerNode> convertToNewInnerNode(cpputils::unique_ref<DataNode> node, const DataNodeLayout &layout, const DataNode &first_child);

protected:
  // The FORMAT_VERSION_HEADER is used to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

  DataNode(DataNodeView block);

  DataNodeView &node();
  const DataNodeView &node() const;
  friend class DataNodeStore;

private:
  DataNodeView _node;

  DISALLOW_COPY_AND_ASSIGN(DataNode);
};

}
}
}


#endif
