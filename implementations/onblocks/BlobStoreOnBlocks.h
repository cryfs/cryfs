#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_

#include "../../interface/BlobStore.h"
#include "messmer/blockstore/interface/BlockStore.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNodeStore;
}

class BlobStoreOnBlocks: public BlobStore {
public:
  static constexpr size_t BLOCKSIZE_BYTES = 4096;

  BlobStoreOnBlocks(std::unique_ptr<blockstore::BlockStore> blockStore);
  virtual ~BlobStoreOnBlocks();

  std::unique_ptr<Blob> create() override;
  std::unique_ptr<Blob> load(const blockstore::Key &key) override;

private:
  std::unique_ptr<datanodestore::DataNodeStore> _nodes;
};

}
}

#endif
