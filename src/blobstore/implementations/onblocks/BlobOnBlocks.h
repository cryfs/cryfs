#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_BLOBONBLOCKS_H_

#include "blobstore/interface/Blob.h"
#include "blockstore/interface/Block.h"

#include <memory>

namespace blobstore {
namespace onblocks {

class BlobOnBlocks: public Blob {
public:
  BlobOnBlocks(std::unique_ptr<blockstore::Block> rootblock);
  virtual ~BlobOnBlocks();

  void *data() override;
  const void *data() const override;

  void flush() override;

  size_t size() const override;

private:
  std::unique_ptr<blockstore::Block> _rootblock;
};

}
}

#endif
