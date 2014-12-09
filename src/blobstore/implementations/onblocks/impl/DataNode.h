#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATANODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATANODE_H_

#include "blockstore/interface/Block.h"

#include <memory>

namespace blobstore {
namespace onblocks {
class DataInnerNode;
class DataLeafNode;

class DataNode {
public:
  virtual ~DataNode();

  static constexpr unsigned char magicNumberInnerNode = 0x01;
  static constexpr unsigned char magicNumberLeaf = 0x02;
  struct NodeHeader {
    unsigned char magicNumber;
  };

  void flush();

  static std::unique_ptr<DataNode> load(std::unique_ptr<blockstore::Block> block);

  static std::unique_ptr<DataInnerNode> initializeNewInnerNode(std::unique_ptr<blockstore::Block> block);
  static std::unique_ptr<DataLeafNode> initializeNewLeafNode(std::unique_ptr<blockstore::Block> block);

protected:
  DataNode(std::unique_ptr<blockstore::Block> block);

  std::unique_ptr<blockstore::Block> _block;

};

}
}

#endif
