#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_
#define BLOBSTORE_IMPLEMENTATIONS_BLOCKED_BLOBSTOREONBLOCKS_H_

#include "blobstore/interface/BlobStore.h"
#include "blockstore/interface/BlockStore.h"

namespace blobstore {
namespace onblocks {

class BlobStoreOnBlocks: public BlobStore {
public:
  //Should be a multiple of 16. The DataNodeView classes have a header of 16 bytes and the block key length (inner data nodes store a list of block keys) is 16 bytes.
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
