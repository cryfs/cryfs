#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_

#include "blobstore/interface/BlobStore.h"
#include "blockstore/interface/BlockStore.h"

namespace blobstore {
namespace onblocks {

class BlobStoreOnBlocks: public BlobStore {
public:
  static constexpr size_t BLOCKSIZE = 4096;

  BlobStoreOnBlocks(std::unique_ptr<blockstore::BlockStore> blockStore);
  virtual ~BlobStoreOnBlocks();

  BlobWithKey create(size_t size) override;
  std::unique_ptr<Blob> load(const Key &key) override;

private:
  std::unique_ptr<blockstore::BlockStore> _blocks;
};

}
}

#endif
