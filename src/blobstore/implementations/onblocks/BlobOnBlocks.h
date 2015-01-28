#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_

#include "blobstore/interface/Blob.h"

#include <memory>

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNode;
}

class BlobOnBlocks: public Blob {
public:
  BlobOnBlocks(std::unique_ptr<datanodestore::DataNode> rootnode);
  virtual ~BlobOnBlocks();

  size_t size() const override;

  void flush() const override;

private:
  std::unique_ptr<datanodestore::DataNode> _rootnode;
};

}
}

#endif
